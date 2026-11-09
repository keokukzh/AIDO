# AIDO-Modelllaufzeiten: Ausführungsplan

**Stand:** 9. November 2026  
**Ziel:** Auf dem Referenz-PC eine nachweisbar lokale Modellkette für Text, Bild, Memory-Suche und Spracheingabe auswählen. Danach erst die AIDO-Integration implementieren.  
**Grundlage:** [Anforderungen](../../REQUIREMENTS.md), [Architektur](../../ARCHITECTURE.md), [Modellinventar](../../MODEL_RUNTIME_INVENTORY.md).

Dieser Plan führt zunächst Prüfungen an vorhandenen Installationen aus. Er legt keine neue Runtime und kein Modell ohne Testergebnis fest. Für die spätere App sind Windows-native Komponenten vorgesehen; Docker/WSL2 bleiben optionale Entwicklungswerkzeuge.

## Ausgangslage

LM Studio enthält eine CUDA-fähige llama.cpp-Engine, mehrere Qwen3.8-GGUF-Dateien, einen separaten Bildprojektor und Nomic Embed Text v1.5. Darüber hinaus liegen auf dem System eigenständige CUDA-13.4-Binaries für `llama-server.exe` (`C:\AI\llamacpp-cuda` Build 11671 und `D:\AI_Models\unsloth\llama.cpp\build\bin\Release` Build 12176) sowie `whisper-server.exe` (`v1.9.5-unsloth.18`) samt `Qwen3.8-27B-UD-Q4_K_M.gguf`, `mmproj-Qwen3.8-27B-BF16.gguf`, `Qwen3.8-27B-DFlash2-Q4_K_M.gguf` und `ggml-small.bin`.

## Arbeitsregeln

- Vor jedem Download vorhandene Modelle, Formate und Lizenzen prüfen; keine bestehende Modellbibliothek verschieben oder bereinigen.
- Keine API-Token in Git, Befehlsausgaben oder Testprotokolle schreiben. Serverauthentifizierung bleibt aktiv.
- Bei jeder Messung genau eine Hauptmodell-Instanz laden und danach entladen; sonst ist der VRAM-Vergleich nicht aussagekräftig.
- Testdaten sind künstlich oder ausdrücklich freigegeben. Screenshots und persönliche Dokumente gehören nicht in den ersten Prüfsatz.
- Pro Aufgabe Ergebnis, Fehlermeldung, Modellkennung, Runtime-Version und Ressourcenwerte dokumentieren.

## Aufgabe 1: Ollama-Zustand auflösen

- [x] Ollama-App, Serverprozess und dessen Modellverzeichnis anhand der aktiven Prozesskonfiguration zuordnen.
- [x] Prüfen, warum eine Abfrage zwei Modelle und ein später gestarteter Server keine Modelle zeigte. Die PowerShell-Sitzung hatte den Benutzerwert von `OLLAMA_MODELS` nicht geerbt; der Benutzerpfad liegt auf `D:\AI_Models\ollama`. Die frühere Zwei-Modell-Liste bleibt einem anderen Serverzustand zugeordnet.
- [x] Entscheiden, welche vorhandenen Ollama-Modelle tatsächlich verfügbar sind; nichts kopieren oder löschen. `qwen3.8:latest` ist wegen fehlendem Blob nicht nutzbar; Ollama bleibt vorläufig außerhalb des Vergleichs.

**Abnahme:** Ein wiederholter Modell-Listentest zeigt beim selben Serverzustand dieselben Modelle. Der verwendete Modellpfad ist dokumentiert. Falls der frühere Serverzustand nicht rekonstruierbar ist, bleibt Ollama vorläufig außerhalb der Vergleichsmessung.

**Ausführungsstand:** Aufgabe 1 mit Einschränkung abgeschlossen. Einzelheiten: [Modellinventar](../../MODEL_RUNTIME_INVENTORY.md).

## Aufgabe 2: Embedding lokal prüfen

- [x] Das vorhandene `text-embedding-nomic-embed-text-v1.5` laden.
- [x] Über einen autorisierten lokalen Client drei harmlose deutsche Testsätze einbetten; drei Vektoren mit je 768 Dimensionen wurden geliefert.
- [x] Mit einer kleinen, vom Team selbst geschriebenen Sammlung prüfen, ob ähnliche Projekttexte näher zueinander liegen als ein themenfremder Text. Kosinusähnlichkeit: 0,8036 gegenüber 0,5959.
- [x] Modell wieder entladen; ein früherer Ladevorgang dauerte laut LM Studio 2,96 s.

**Abnahme:** Die einmalige lokale Funktionsprobe lieferte drei 768-dimensionale Vektoren; der erwartete Vergleich ähnlicher und unähnlicher Sätze gelang.

**Ausführungsstand:** Der autorisierte Embedding-Funktionstest ist bestanden. Der temporär bereitgestellte Zugang wurde nicht gespeichert.

## Aufgabe 3: Bild-Eingang der konkreten Qwen-Datei prüfen

- [x] Die gewählte Standard-Q4-Datei `D:\AI_Models\llm\Qwen3.8-27B-UD-Q4_K_M.gguf` und den vorhandenen passenden Projektor `D:\AI_Models\llm\assets\mmproj-Qwen3.8-27B-BF16.gguf` in `llama-server.exe` (`v0.9.0-dev`, Build 12176) zuordnen.
- [x] Mit einem künstlich erzeugten Testbild (256×128 PNG, linke Hälfte rot, rechte Hälfte blau) eine konkrete Bildfrage stellen.
- [x] Prüfen, ob der Aufruf wirklich Bilddaten enthält und die Antwort ohne Bild anders ausfällt.
- [x] Bei fehlender Projektorunterstützung in der LM-Studio-Chat-GUI auf den direkten lokalen `llama-server`-Runtime-Pfad wechseln und diesen als primäre AIDO-Runtime festlegen.

**Abnahme:** Bildinhalt wird korrekt erkannt (*„Die linke Hälfte des Bildes ist rot und die rechte Hälfte ist blau.“* vs. Kontrollantwort ohne Bild: *„Es wurde kein Bild in dieser Unterhaltung hochgeladen, daher kann ich die Farben nicht bestimmen.“*); Modell-, Projektor- und Runtime-Version stehen im Protokoll.

**Ausführungsstand:** Abgeschlossen am 9. November 2026. Über `llama-server.exe` (Build 12176) mit `--mmproj` und `--spec-type draft-dflash` wurde das Testbild beim Kaltstart inklusive Modellladen in `9,41 s` (`102,70 Token/s` Generierung) fehlerfrei ausgewertet. Siehe [Modellinventar](../../MODEL_RUNTIME_INVENTORY.md).

## Aufgabe 4: Spracheingabe und optionale Ausgabe auswählen

- [x] Vorhandene lokale ASR- und TTS-Runtimes inventarisieren (`whisper-server.exe` v1.9.5-unsloth.18 sowie Windows `System.Speech` mit `Microsoft Hedda Desktop`).
- [x] Für Spracheingabe `ggml-small.bin` mit deutschem, freigegebenem Testaudio prüfen; Transkript, Latenz und Offline-Verhalten auf `127.0.0.1:18080` erfassen.
- [x] Sprachausgabe separat für lokale synthetische Tests prüfen; weitergehende neuronale TTS-Bewertung folgt bei Ausbau der Voice-Oberfläche.

**Abnahme:** Spracheingabe liefert ein verständliches deutsches Transkript ohne externe Anfrage. Embeddings werden ausschließlich für Suche und Memory verwendet.

**Ausführungsstand:** Abgeschlossen am 9. November 2026. Durch Einbinden des lokalen CUDA-13-DLL-Verzeichnisses (`C:\AI\llamacpp-cuda`) erkannte `whisper-server.exe` die RTX 5090 (`CUDA0`, `487,01 MB` Modellgröße im VRAM, `+1.150 MiB` System-VRAM) und transkribierte ein 6,9 Sekunden langes deutsches WAV-Testaudio in **`0,341 s`** (*„Hallo IDO, bitte führe einen schnellen Systemcheck durch und zeige mir den Speicherverbrauch.“*).

## Aufgabe 5: Hauptmodell und Laufzeit vergleichen

- [x] `Qwen3.8-27B-UD-Q4_K_M` + `mmproj-Qwen3.8-27B-BF16` + `Qwen3.8-27B-DFlash2-Q4_K_M` auf den lokalen `llama-server.exe`-Builds (Build 11671 und Build 12176) für Text-, Vision- und Aufwachaufgaben messen.
- [x] Kaltstart (`9,41 s` inkl. Vision), warme Antwortrate (**`151,54 Token/s`** mit DFlash2 auf Build 12176; `130,39 Token/s` auf Build 11671), VRAM-Belegung (`25.544 MiB` geladen bei 32k Kontext = `+23.237 MiB`) und automatisches Entladen nach Leerlauf (`--sleep-idle-seconds 10` → Rückgang auf `2.830 MiB`, Wake-up in `8,26 s`) dokumentieren.
- [x] Direkten `llama-server.exe`-Router-Modus (`127.0.0.1:18434`) wegen nativer Projektor-, DFlash2- und Idle-Sleep-Unterstützung als primäre Runtime für den AIDO-Prototyp festlegen.
- [x] Thinking-Ausgabe über `--chat-template-kwargs {"reasoning_effort":"low"}` für schnelle Desktop-Antworten begrenzen.

**Abnahme:** Die Messwerte begründen `llama-server.exe` (Build 12176) mit `Qwen3.8-27B-UD-Q4_K_M` + `mmproj-BF16` + `DFlash2` als primäre lokale Runtime für den Referenz-PC.

## Aufgabe 6: Installationsmanifest und nächster Softwareblock

- [x] Für jede gewählte Funktion Modellrolle, Dateipfad, Größe, SHA-256-Prüfsumme, Runtime und VRAM-Bedarf in [Modellinventar](../../MODEL_RUNTIME_INVENTORY.md) erfassen.
- [x] Wiederverwendung vorhandener Installationen ohne Kopieren großer Modelldateien festlegen.
- [x] Den ersten Windows-Prototyp für das End-to-End-Szenario **System-Gesundheitscheck & Fast Cleanup nach Nutzerfreigabe** (Tauri 2 + Permission Gateway + Windows Bridge + SQLite/FTS5 Memory) umsetzen.

**Abnahme:** Alle Aufgaben (1 bis 6) dieses Modell-Validierungsplans sind abgeschlossen und im Repository dokumentiert.
