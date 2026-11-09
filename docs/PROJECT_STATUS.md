# Projektstatus

**Stand: 9. November 2026.**

## Verifiziert

- Das GitHub-Repository `keokukzh/AIDO` war vor dieser Dokumentationsinitialisierung privat und leer. Der verbundene GitHub-Zugang meldete Schreib- und Administratorberechtigung.
- Die AIDO-Interviews und die Produktplanung vom 9. November 2026 enthalten Entscheidungen zur Zielgruppe, Windows 11, lokal bevorzugter KI, App-Erkennung, Berechtigungsmodi, proaktiven Vorschlägen und schrittweiser Entwicklung.
- Das Repository enthält weiterhin nur Grundlagendokumentation, nun mit einer Einordnung des nachgereichten Docker-Plans und des früheren Produktgedankens.
- Ein weiterer punktueller Check fand Ollama unter Windows (Client 0.40.14, Server 0.40.11), zwei gelistete Modelle (`GameDev-Qwen3.8-uncensored:latest`, 16 GB, und `qwen2.5:14b`, 9 GB) sowie eine LM-Studio-Installation. `llama-server` und `llama-cli` wurden weder im Windows-PATH noch im Ubuntu-PATH gefunden; eine separate Installation an anderem Ort ist damit nicht ausgeschlossen. Ein späterer `ollama show`-Versuch scheiterte, weil der lokale Server zu diesem Zeitpunkt nicht erreichbar war. Bildfähigkeit der installierten Qwen-Ableitung wurde nicht getestet.
- Ein lesender Ist-Check bestätigte Docker Desktop 29.9.2 und eine RTX 5090. Docker war in Ubuntu mangels WSL-Integration nicht nutzbar; der im Altplan genannte Sandbox-Ordner fehlte. Details: [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md).

Die Modelllaufzeiten und ersten Kurztests sind in [MODEL_RUNTIME_INVENTORY.md](MODEL_RUNTIME_INVENTORY.md) festgehalten. LM Studio enthält bereits llama.cpp-Engines und ein Text-Embedding-Modell. Ein lokaler Qwen-Texttest gelang; Embedding-API und Bild-Eingang sind noch nicht erfolgreich getestet.

## Nicht nachgewiesen

Eine laufende AIDO-App, ein Installer, ein Adapter, ein lokales Modell innerhalb von AIDO, ein Memory-Dienst, ein Sicherheits-Gateway oder Produkttests sind in diesem Repository nicht nachgewiesen. Die hier genannten punktuellen Systembeobachtungen ersetzen keine vollständige Inventur des privaten Referenzsystems.

Der Docker-Anhang wurde nachgereicht. Er ist ein älterer Plan, kein Beleg für ausgeführte Einrichtung. WSL-Integration, GPU-Passthrough, Compose-Dateien und Backups sind nicht als funktionsfähig nachgewiesen.

## Nächste verifizierbare Arbeit

Zuerst die Entwicklungsumgebung gezielt vervollständigen und absichern, dann einen kleinen Windows-11-Prototyp für freigegebene Aktionen und deren Protokollierung bauen. Danach App-Erkennung und einen einzelnen überprüfbaren Adapter ergänzen. Die weiteren Phasen stehen in [ROADMAP.md](ROADMAP.md).

Dieser Status wird nur anhand realer Repository-Artefakte, ausführbarer Ergebnisse und dokumentierter Tests fortgeschrieben.
