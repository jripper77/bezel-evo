$inputText = [Console]::In.ReadToEnd()
try {
    $payload = $inputText | ConvertFrom-Json
    if ($payload.session_id -match '^[a-zA-Z0-9-]{1,128}$') {
        $record = @{ session_id = $payload.session_id; rate_limits = $payload.rate_limits }
        $destination = Join-Path $bezelOutput ($payload.session_id + '.json')
        $temporary = Join-Path $bezelOutput ([Guid]::NewGuid().ToString() + '.tmp')
        try {
            [IO.File]::WriteAllText($temporary, ($record | ConvertTo-Json -Depth 8 -Compress), [Text.UTF8Encoding]::new($false))
            if ([IO.File]::Exists($destination)) { [IO.File]::Replace($temporary, $destination, [NullString]::Value) }
            else { [IO.File]::Move($temporary, $destination) }
        } finally {
            if ([IO.File]::Exists($temporary)) { [IO.File]::Delete($temporary) }
        }
    }
} catch { }
if ($PassThrough) {
    [Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
    [Console]::Out.Write($inputText)
}
