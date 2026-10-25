//! Bounded engine messages. JSON carries document structure; assets/images
//! remain binary. Neither peer can pass device protocol bytes to the engine.
use super::*;
use bezel_themes::dto::ThemeDto;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    #[serde(default)]
    pub debug_overlay: Option<crate::performance::Corner>,
    pub ping_host: String,
    #[serde(default)]
    pub audio_capture: bool,
    pub presentmon_path: Option<String>,
    pub ffmpeg: Option<String>,
}

/// The element a canvas trial holds still and the share of its scale.
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrialHold {
    pub element: u32,
    pub fraction: f64,
}

pub const MAX_JSON: usize = 4 * 1024 * 1024;
pub const MAX_METADATA: usize = 65536;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMessage {
    pub runtime: u8,
    pub scene: String,
    pub op: RuntimeOp,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "command", deny_unknown_fields)]
pub enum RuntimeOp {
    SensorRestarted,
    Options {
        options: Options,
    },
    Draft {
        revision: u64,
        saved: bool,
    },
    Cards {
        on: bool,
    },
    Live {
        on: bool,
    },
    Image {
        after: u64,
        max: u32,
        cards: bool,
    },
    Inspect {
        element: u32,
        max: u32,
        motion: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trial_ms: Option<u32>,
    },
    Measure {
        element: u32,
    },
    /// The whole scene with trial inputs for the canvas's trial values, at
    /// `ms` of the trial; with `hold`, one element's sensor held at
    /// `fraction` of its scale instead. Never shown live, never saved.
    Trial {
        max: u32,
        motion: bool,
        #[serde(default)]
        ms: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hold: Option<TrialHold>,
    },
    Thumbnail,
    /// Tries a profile cover on every live screen without changing a theme
    /// (the scene named is ignored); answers `{"screens": n}`.
    PreviewCover {
        transition: crate::profiles::TransitionSettings,
        /// The theme to try on each screen, by screen key (the profile
        /// selected in the manager, saved or not).
        #[serde(default)]
        themes: BTreeMap<String, String>,
    },
    /// A thirty-second trial on live displays, without editing saved profiles.
    TryThemes {
        themes: BTreeMap<String, String>,
        profile: Option<String>,
    },
    Sensors {
        listed: Option<Vec<String>>,
    },
    /// What the profiles do now (the scene named is ignored): the active
    /// profile and why, and which rules are true.
    ProfileStatus,
}
#[derive(Serialize, Deserialize)]
pub struct Reply {
    pub value: Value,
    pub error: Option<String>,
}
pub struct Image {
    pub render_millis: f64,
    pub sequence: u64,
    pub next_ms: u32,
    pub frame: Option<Frame>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    theme: ThemeDto,
    assets: Option<Vec<(String, u32)>>,
}
pub fn request(scene: &str, op: RuntimeOp) -> Result<Vec<u8>> {
    serde_json::to_vec(&RuntimeMessage {
        runtime: 2,
        scene: scene.into(),
        op,
    })
    .map_err(|e| invalid(&e.to_string()))
}
/// Expand a bounded JSON reply carried outside the small metadata envelope.
pub fn unpack_reply(reply: (Vec<u8>, Vec<u8>)) -> Result<(Vec<u8>, Vec<u8>)> {
    if reply.0.len() > MAX_METADATA {
        return Err(invalid("reply metadata size"));
    }
    let envelope: Value =
        serde_json::from_slice(&reply.0).map_err(|_| invalid("invalid runtime reply"))?;
    if envelope["encoding"] == "json" {
        if reply.1.len() > MAX_JSON {
            return Err(invalid("runtime reply exceeds limits"));
        }
        Ok((reply.1, Vec::new()))
    } else {
        Ok(reply)
    }
}
pub fn is_request(bytes: &[u8]) -> bool {
    bytes.len() <= MAX_METADATA
        && serde_json::from_slice::<RuntimeMessage>(bytes).is_ok_and(|r| r.runtime == 2)
}
/// Assets are included only when their content revision changes.
pub fn document(theme: &Theme, assets: Option<&BTreeMap<AssetRef, Vec<u8>>>) -> Result<Vec<u8>> {
    let manifest = Manifest {
        theme: ThemeDto::from(theme),
        assets: assets
            .map(|a| {
                a.iter()
                    .map(|(k, v)| {
                        Ok((
                            k.0.clone(),
                            u32::try_from(v.len()).map_err(|_| invalid("asset too large"))?,
                        ))
                    })
                    .collect::<Result<_>>()
            })
            .transpose()?,
    };
    let json = serde_json::to_vec(&manifest).map_err(|e| invalid(&e.to_string()))?;
    let total = 4usize
        .checked_add(json.len())
        .and_then(|n| n.checked_add(assets.map_or(0, |a| a.values().map(Vec::len).sum())))
        .ok_or_else(|| invalid("document overflow"))?;
    if json.len() > MAX_JSON || total > MAX_DOCUMENT {
        return Err(invalid("document exceeds wire limits"));
    }
    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(&(json.len() as u32).to_le_bytes());
    bytes.extend(json);
    if let Some(assets) = assets {
        for bytes_of_asset in assets.values() {
            bytes.extend_from_slice(bytes_of_asset);
        }
    }
    Ok(bytes)
}
fn read_document(engine: &Engine, key: &str, bytes: &[u8]) -> Result<Document> {
    if bytes.len() > MAX_DOCUMENT || bytes.len() < 4 {
        return Err(invalid("document size"));
    }
    let len = u32::from_le_bytes(
        bytes[..4]
            .try_into()
            .map_err(|_| invalid("document header"))?,
    ) as usize;
    if len > MAX_JSON || len > bytes.len() - 4 {
        return Err(invalid("document metadata size"));
    }
    let manifest: Manifest =
        serde_json::from_slice(&bytes[4..4 + len]).map_err(|e| invalid(&e.to_string()))?;
    let theme = Theme::try_from(&manifest.theme).map_err(|e| invalid(&e.0))?;
    let mut rest = &bytes[4 + len..];
    let assets = if let Some(entries) = manifest.assets {
        if entries.len() > 4096 {
            return Err(invalid("too many assets"));
        }
        let mut assets = BTreeMap::new();
        for (name, len) in entries {
            let len = len as usize;
            let (data, left) = rest
                .split_at_checked(len)
                .ok_or_else(|| invalid("truncated asset"))?;
            let asset = AssetRef(name);
            bezel_themes::native::safe_asset_path(&asset).map_err(|e| invalid(&e.to_string()))?;
            if assets.contains_key(&asset) {
                return Err(invalid("duplicate asset"));
            }
            assets.insert(asset, data.to_vec());
            rest = left;
        }
        assets
    } else {
        engine
            .scenes
            .get(key)
            .ok_or_else(|| invalid("missing initial assets"))?
            .runtime
            .assets()
            .clone()
    };
    if !rest.is_empty() {
        return Err(invalid("trailing document data"));
    }
    let document = Document { theme, assets };
    document.validate()?;
    Ok(document)
}
/// Reply decoding is shared by Studio's remote and in-process transports.
pub fn response(metadata: &[u8]) -> Result<Value> {
    if metadata.len() > MAX_JSON {
        return Err(invalid("reply metadata size"));
    }
    let reply: Reply = serde_json::from_slice(metadata).map_err(|e| invalid(&e.to_string()))?;
    if let Some(error) = reply.error {
        return Err(invalid(&error));
    }
    Ok(reply.value)
}
pub(crate) fn image(frame: Option<Frame>, sequence: u64, next_ms: u32) -> (Value, Vec<u8>) {
    match frame {
        Some(frame) => (
            json!({"width":frame.size().width,"height":frame.size().height,"sequence":sequence,"nextMs":next_ms}),
            frame.as_rgba().to_vec(),
        ),
        None => (
            json!({"width":0,"height":0,"sequence":sequence,"nextMs":next_ms}),
            Vec::new(),
        ),
    }
}
/// Validates bounded image dimensions before copying any pixels into a frame.
pub fn frame(metadata: &Value, pixels: Vec<u8>) -> Result<Option<Frame>> {
    let width = metadata["width"]
        .as_u64()
        .ok_or_else(|| invalid("image width"))?;
    let height = metadata["height"]
        .as_u64()
        .ok_or_else(|| invalid("image height"))?;
    if width == 0 && height == 0 && pixels.is_empty() {
        return Ok(None);
    }
    if width == 0
        || height == 0
        || width > 2320
        || height > 2320
        || width.checked_mul(height).and_then(|n| n.checked_mul(4)) != Some(pixels.len() as u64)
    {
        return Err(invalid("image bounds"));
    }
    Frame::from_rgba(
        bezel_core::domain::geometry::Size::new(width as u32, height as u32),
        pixels,
    )
    .map(Some)
    .ok_or_else(|| invalid("image shape"))
}
fn dispatch_inner(
    engine: &mut Engine,
    request: RuntimeMessage,
    payload: &[u8],
) -> Result<(Value, Vec<u8>)> {
    let key = &request.scene;
    if !matches!(request.op, RuntimeOp::Draft { .. } | RuntimeOp::Thumbnail) && !payload.is_empty()
    {
        return Err(invalid("unexpected runtime payload"));
    }
    if matches!(
        request.op,
        RuntimeOp::Draft { .. }
            | RuntimeOp::Inspect { .. }
            | RuntimeOp::Trial { .. }
            | RuntimeOp::Measure { .. }
            | RuntimeOp::Cards { .. }
    ) {
        engine.editing_profile(key);
    }
    match request.op {
        RuntimeOp::SensorRestarted => {
            light(LightEvent::SensorRestart, 0, 0.0, "reader");
            Ok((Value::Null, Vec::new()))
        }
        RuntimeOp::Cards { on } => {
            let scene = engine
                .scenes
                .get_mut(key)
                .ok_or_else(|| invalid("unknown scene"))?;
            if scene.cards != on {
                scene.cards = on;
                scene.runtime.automatic_cards(on);
                scene.dirty = true;
                engine.wake.notify();
            }
            Ok((Value::Null, Vec::new()))
        }
        RuntimeOp::Options { options } => {
            engine.options(options)?;
            Ok((Value::Null, Vec::new()))
        }
        RuntimeOp::Draft { revision, saved } => {
            let doc = read_document(engine, key, payload)?;
            engine.document(key, revision, doc, saved)?;
            Ok((Value::Null, Vec::new()))
        }
        RuntimeOp::Live { on } => {
            engine.live(key, on)?;
            Ok((Value::Null, Vec::new()))
        }
        RuntimeOp::TryThemes { themes, profile } => {
            if themes.len() > MAX_SCENES
                || themes
                    .iter()
                    .any(|(key, path)| key.len() > 256 || path.len() > 32768)
                || profile.as_ref().is_some_and(|id| id.len() > 160)
            {
                return Err(invalid("invalid trial themes"));
            }
            let screens = engine.try_themes(&themes, profile)?;
            Ok((json!({ "screens": screens }), Vec::new()))
        }
        RuntimeOp::PreviewCover { transition, themes } => {
            if themes.len() > MAX_SCENES
                || themes.iter().any(|(k, v)| k.len() > 256 || v.len() > 32768)
            {
                return Err(invalid("invalid preview themes"));
            }
            let covered = engine.preview_cover(&transition, &themes)?;
            Ok((json!({ "screens": covered }), Vec::new()))
        }
        RuntimeOp::Image { after, max, .. } => {
            let next = engine.image(key, after, max)?;
            let (mut value, pixels) = image(next.frame, next.sequence, next.next_ms);
            value["renderMillis"] = json!(next.render_millis);
            Ok((value, pixels))
        }
        RuntimeOp::Inspect {
            element,
            max,
            motion,
            trial_ms,
        } => {
            let (frame, next) =
                engine.inspect_trial(key, ElementId(element), max, motion, trial_ms)?;
            Ok(image(
                Some(frame),
                0,
                next.map_or(u32::MAX, |d| {
                    d.as_millis().min(u128::from(u32::MAX - 1)) as u32
                }),
            ))
        }
        RuntimeOp::Trial {
            max,
            motion,
            ms,
            hold,
        } => {
            let hold = hold.map(|h| crate::inspector_trial::Hold {
                element: ElementId(h.element),
                fraction: h.fraction,
            });
            let frame = engine.trial_canvas(key, max, motion, ms, hold)?;
            Ok(image(
                Some(frame),
                0,
                crate::inspector_trial::FRAME_MS as u32,
            ))
        }
        RuntimeOp::Measure { element } => {
            let value = engine
                .measure(key, ElementId(element))?
                .map(|b| json!({"x":b.x,"y":b.y,"width":b.width,"height":b.height}));
            Ok((json!(value), Vec::new()))
        }
        RuntimeOp::Thumbnail => {
            let doc = read_document(engine, key, payload)?;
            let mut runtime = ThemeRuntime::new(doc.theme, doc.assets, engine.language);
            if let Some(sensors) = &mut engine.thumbnail_sensors {
                runtime.use_catalog(&sensors.catalog()?);
                let count = runtime
                    .theme()
                    .history_lengths()
                    .iter()
                    .map(|(_, n)| *n)
                    .max()
                    .unwrap_or(2)
                    .clamp(2, 600);
                for _ in 0..count {
                    runtime.sample(sensors.as_mut())?;
                }
            } else {
                runtime.use_catalog(&engine.catalog);
            }
            // Thumbnail snapshots are isolated and never sample hardware or alter a live graph.
            engine.renderer.set_scene(u64::MAX);
            engine.renderer.set_motion(false);
            let (frame, _) =
                runtime.preview(engine.renderer.as_mut(), (engine.clock)(), Duration::ZERO)?;
            Ok(image(
                Some(element_preview::fit_within(frame, 360)),
                0,
                u32::MAX,
            ))
        }
        RuntimeOp::ProfileStatus => Ok((
            serde_json::to_value(engine.profile_status()).unwrap_or(Value::Null),
            Vec::new(),
        )),
        RuntimeOp::Sensors { listed } => {
            if let Some(keys) = listed {
                let wanted = keys
                    .iter()
                    .filter_map(bezel_core::domain::sensor::SensorKey::new)
                    .collect();
                if let Some(scene) = engine.scenes.get_mut(key) {
                    scene.runtime.want_also(wanted);
                }
                engine.schedule_sensors();
            }
            let scene = engine.scenes.get(key);
            let activity=scene.map(|s|s.runtime.snapshot()).filter(|s|s.activity_available || s.media_available).map(|s| {
                let media:Vec<_>=s.media.iter().map(|m|json!({"source":m.source,"title":m.title,"artist":m.artist,"playing":m.playing})).collect();
                json!({"available":s.activity_available,"foreground":s.foreground,"applications":s.applications,"media":media})
            });
            let video = scene.and_then(|s| match s.runtime.video() {
                VideoState::NoVideo => None,
                VideoState::NotStarted => Some(json!({"state":"notStarted","path":null})),
                VideoState::OnDevice(p) => Some(json!({"state":"onDevice","path":p.to_string()})),
                VideoState::StoredMissing(p) => {
                    Some(json!({"state":"storedMissing","path":p.to_string()}))
                }
                VideoState::VideoMissing(p) => {
                    Some(json!({"state":"missing","path":p.path.to_string()}))
                }
                VideoState::Host => Some(json!({"state":"host","path":null})),
                VideoState::NoConverter { .. } => Some(json!({"state":"noConverter","path":null})),
                VideoState::NoPlayback => Some(json!({"state":"noPlayback","path":null})),
            });
            let catalog:Vec<_>=engine.catalog.iter().map(|s|json!({"key":s.key.to_string(),"category":s.category.slug(),"label":s.label,"quantity":s.quantity.slug(),"source":s.source})).collect();
            let mut readings = BTreeMap::new();
            if let Some(scene) = scene {
                for (key, r) in scene.runtime.snapshot().iter() {
                    use bezel_core::domain::sensor::Reading;
                    let value = match r {
                        Reading::Value(v) => json!({"value":v}),
                        Reading::Text(t) => json!({"text":t}),
                        Reading::Unavailable(e) => json!({"unavailable":e}),
                    };
                    readings.insert(key.to_string(), value);
                }
            }
            Ok((
                json!({"sampleAtMs":engine.sensors.latest().at.as_millis() as u64,"catalog":catalog,"readings":readings,"fps":scene.map(|s|s.frames as f64/engine.origin.elapsed().as_secs_f64().max(0.001)),"error":scene.and_then(|s|s.error.clone()),"video":video,"activity":activity}),
                Vec::new(),
            ))
        }
    }
}
pub fn dispatch(
    engine: &mut Engine,
    metadata: &[u8],
    payload: &[u8],
) -> Result<(Vec<u8>, Vec<u8>)> {
    let result = (|| {
        if metadata.len() > MAX_METADATA || payload.len() > MAX_DOCUMENT {
            return Err(invalid("runtime message too large"));
        }
        let request: RuntimeMessage =
            serde_json::from_slice(metadata).map_err(|e| invalid(&e.to_string()))?;
        if request.runtime != 2 {
            return Err(invalid("unsupported runtime version"));
        }
        dispatch_inner(engine, request, payload)
    })();
    let (reply, pixels) = match result {
        Ok((value, pixels)) => (Reply { value, error: None }, pixels),
        Err(e) => (
            Reply {
                value: Value::Null,
                error: Some(e.to_string()),
            },
            Vec::new(),
        ),
    };
    let metadata = serde_json::to_vec(&reply).map_err(|e| invalid(&e.to_string()))?;
    if metadata.len() > MAX_METADATA {
        // Large catalogs are bounded binary JSON, not pipe metadata.
        // A value beyond that ceiling is a request error, not a broken channel.
        if metadata.len() <= MAX_JSON && pixels.is_empty() {
            return Ok((
                br#"{"value":null,"error":null,"encoding":"json"}"#.to_vec(),
                metadata,
            ));
        }
        return Ok((
            br#"{"value":null,"error":"runtime reply exceeds limits"}"#.to_vec(),
            Vec::new(),
        ));
    }
    Ok((metadata, pixels))
}

/// Typed catalog decoding shared by the UI's readings and catalog adapters.
pub fn catalog(value: &Value) -> Vec<SensorInfo> {
    use bezel_core::domain::sensor::{Category, Quantity, SensorKey};
    let quantities = [
        Quantity::Percent,
        Quantity::Celsius,
        Quantity::Megahertz,
        Quantity::Watts,
        Quantity::Volts,
        Quantity::Amperes,
        Quantity::Rpm,
        Quantity::Bytes,
        Quantity::BytesPerSecond,
        Quantity::Seconds,
        Quantity::Number,
        Quantity::Text,
    ];
    value["catalog"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|s| {
            Some(SensorInfo {
                key: SensorKey::new(s["key"].as_str()?)?,
                category: Category::ALL
                    .into_iter()
                    .find(|c| Some(c.slug()) == s["category"].as_str())?,
                quantity: quantities
                    .into_iter()
                    .find(|q| Some(q.slug()) == s["quantity"].as_str())?,
                label: s["label"].as_str()?.into(),
                source: s["source"].as_str()?.into(),
            })
        })
        .collect()
}
/// Numeric/text readings retain the renderer's native units.
pub fn snapshot(value: &Value) -> bezel_core::domain::sensor::Snapshot {
    use bezel_core::domain::sensor::{Reading, SensorKey, Snapshot};
    let mut snapshot = Snapshot::default();
    if let Some(readings) = value["readings"].as_object() {
        for (key, v) in readings {
            if let Some(key) = SensorKey::new(key) {
                let reading = if let Some(v) = v["value"].as_f64() {
                    Reading::Value(v)
                } else if let Some(t) = v["text"].as_str() {
                    Reading::Text(t.into())
                } else {
                    Reading::Unavailable(v["unavailable"].as_str().unwrap_or("unavailable").into())
                };
                snapshot.insert(key, reading);
            }
        }
    }
    snapshot
}
