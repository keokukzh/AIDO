# Projektstatus

**Stand: 9. November 2026.** Dieser Status trennt Repository-Artefakte von punktuellen Beobachtungen am Referenz-PC.

## Repository

- `keokukzh/AIDO` ist laut GitHub-Metadaten privat; der verbundene Zugang hat Schreib- und Administratorberechtigung.
- Das Repository enthält Produktgrundlagen, Entwicklungsplanung und Prüfprotokolle. Eine AIDO-App oder ausführbarer Produktcode ist hier bislang nicht nachgewiesen.
- Der historische Docker-Anhang ist als frühere Setup-Idee eingeordnet, nicht als ausgeführte Installation. Windows-native Komponenten sind für die spätere Nutzer-App vorgesehen; Docker/WSL2 sind optionale Entwicklungswerkzeuge.

## Beobachtungen am Referenz-PC

- LM Studio 0.4.28 ist installiert. Sein lokaler Server lief zum Prüfzeitpunkt auf `127.0.0.1:1234` mit erforderlicher API-Authentifizierung. Eine CUDA-fähige llama.cpp-Engine, mehrere Qwen3.8-GGUF-Dateien, ein separater Bildprojektor und Nomic Embed Text v1.5 sind vorhanden.
- Ein einzelner lokaler Qwen-Textprompt wurde beantwortet. Die dabei gemessene Tokenrate ist kein Vergleichsbenchmark.
- Ein erster Embedding-Aufruf ohne Token wurde abgewiesen. Ein späterer autorisierter Test lieferte drei Vektoren mit je 768 Dimensionen; ein einfacher Vergleich deutscher Sätze ergab 0,8036 für verwandte und 0,5959 für themenfremde Texte. Das belegt eine Funktionsprobe, keine Suchqualität auf realen AIDO-Daten. Der im Chat offengelegte Zugang muss in LM Studio widerrufen werden; er steht nicht in Repository-Dateien.
- Ollamas Benutzer-Modellpfad wurde als `D:\\AI_Models\\ollama` identifiziert. Ein temporärer Server zeigte wiederholt dieselbe Modellliste, aber `qwen3.8:latest` ließ sich wegen eines fehlenden Blobs nicht öffnen. Ollama ist daher vorläufig kein belastbarer Vergleichskandidat.
- Ein `whisper-server.exe` ist vorhanden; in den gezielt geprüften Verzeichnissen wurde kein zugehöriges Sprachmodell gefunden. Spracheingabe wurde nicht getestet.
- Docker Desktop 29.9.2 und eine RTX 5090 wurden punktuell festgestellt. Docker war in Ubuntu mangels WSL-Integration nicht nutzbar; GPU-Passthrough und der im Altplan genannte Sandbox-Ordner sind nicht nachgewiesen. Details: [Entwicklungsumgebung](DEVELOPMENT_SETUP.md).

## Noch nicht nachgewiesen

Die installierten Qwen-Dateien wurden von LM Studio mit `vision: false` gelistet. Der vorhandene Projektor und das offizielle bildfähige Qwen-Modell belegen daher noch keinen funktionierenden Bild-Eingang dieser konkreten Ableitung. Es fehlen ein Bildtest mit passender Projektorzuordnung, ein deutscher ASR-Test, Offline- und Ressourcenmessungen, Reproduzierbarkeit der Embeddings sowie die AIDO-Integration. Ein Installer, Memory-Dienst, Sicherheits-Gateway, App-Adapter und Produkttests sind nicht implementiert oder nachgewiesen.

## Nächster Schritt

Den Bild-Eingang der konkreten Qwen-Q4-Datei mit dem vorhandenen Projektor und einem künstlichen Testbild prüfen. Danach lokale Spracheingabe inventarisieren und Hauptmodellvarianten unter gleichen Bedingungen vergleichen. Der genaue Ablauf und offene Abnahmekriterien stehen im [Ausführungsplan](superpowers/plans/2026-11-09-aido-model-runtime-validation.md); Rohbefunde im [Modellinventar](MODEL_RUNTIME_INVENTORY.md).
