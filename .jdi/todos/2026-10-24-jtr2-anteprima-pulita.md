# JTR2 Black e cambio tema con anteprima pulita — 24 ottobre 2026

## Richiesta e chiarimento

L'utente segnala JTR2 Black non funzionante e il vecchio tema Pixel ancora visibile sotto gli oggetti importati. Chiarimento esplicito: vuole un tema nuovo/pulito nel canvas, non nuove schede o finestre.

## Riscontri

- Il file originale sul Desktop viene importato: 67 elementi, 8 asset, tela 1920×480.
- Il rendering CLI con sensori demo produce correttamente JTR2 Black. Evidenza: target/logs/jtr2-open-document/jtr2-black.png.
- Il tema include un video MP4 e un poster. Il log Light durante il problema ripete ScreenError/scene; la vecchia versione nasconde la causa dietro «scene update failed».
- Il motore interrompeva anche il rendering dell'editor quando falliva la preparazione del video/display; manteneva lo snapshot precedente. Il canvas non veniva svuotato se il nuovo tema aveva le stesse dimensioni.
- La specifica causa hardware/video della sessione dell'utente rimane da leggere con la nuova diagnostica; non è stata confermata direttamente.

## Correzioni

- Runtime ab6dac2: invalida lo snapshot quando cambia il documento; avvio video/display separato dal rendering di poster e oggetti; errori con causa conservata; un errore non invia frame al display non pronto. L'adapter richiede un frame completo dopo un nuovo documento anche se i pixel coincidono.
- Studio 6d7271e7: attende richieste in uscita prima di aprire/importare/creare; svuota il canvas su load e scarta le risposte di una precedente generazione del documento.
- Il comportamento di salvataggio e la richiesta relativa alle modifiche non salvate restano quelli esistenti.

## Verifiche

- Runtime: 70 test verdi e prova locale ignorata per default eseguita separatamente su JTR2 Black, verde. Include regressione con cache video non scrivibile e verifica del nuovo snapshot.
- Studio: 233 test e un doctest verdi; hardware ignorato. Nuova regressione del cursore su immagini identiche.
- Clippy runtime/Studio e fmt mirati verdi.
- UI: 30 unitari verdi.
- E2E: 20 casi esistenti scenarios-library, 4 nuovi casi di import con snapshot non pronto, 4 multi-screen verdi (28 casi distinti).
- Computer Use della sessione non disponibile (native pipe assente); nessuna modifica del documento aperto tramite automazione.

## Consegna

Build locale 0.2.2 avviata con build-evo.ps1. Studio PID 10876 e Light PID 49548 erano aperti da target/release; richiesto all'utente di salvare e chiudere prima di sostituire gli eseguibili, secondo AGENTS.md. Nessun processo terminato forzatamente. Nessun push/installazione/pubblicazione. Prova visiva finale JTR2 Black ancora da fare dopo il riavvio della build nuova.