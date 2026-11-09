# AIDO-Modelllaufzeiten: Ausführungsplan

**Stand:** 9. November 2026  
**Ziel:** Auf dem Referenz-PC eine nachweisbar lokale Modellkette für Text, Bild, Memory-Suche und Spracheingabe auswählen. Danach erst die AIDO-Integration implementieren.  
**Grundlage:** [Anforderungen](../../REQUIREMENTS.md), [Architektur](../../ARCHITECTURE.md), [Modellinventar](../../MODEL_RUNTIME_INVENTORY.md).

Dieser Plan führt zunächst Prüfungen an vorhandenen Installationen aus. Er legt keine neue Runtime und kein Modell ohne Testergebnis fest. Für die spätere App sind Windows-native Komponenten vorgesehen; Docker/WSL2 bleiben optionale Entwicklungswerkzeuge.

## Ausgangslage

LM Studio enthält eine CUDA-fähige llama.cpp-Engine, mehrere Qwen3.8-GGUF-Dateien, einen separaten Bildprojektor und Nomic Embed Text v1.5. Ein Qwen-Texttest gelang. Für Embeddings verlangt der lokale LM-Studio-Server ein API-Token. Die gelisteten Qwen-Dateien tragen `vision: false`; Bild-Eingang ist offen. Ollamas Modellliste war zwischen Serverzuständen unterschiedlich. Es wurden keine neuen Modelle installiert.

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

**Ausführungsstand:** Aufgabe 1 abgeschlossen mit Einschränkung. Embedding-API-Test durch Tokenpflicht und blockierten separaten Testserver offen. Bild- und Audiotests offen. Einzelheiten: [Modellinventar](../../MODEL_RUNTIME_INVENTORY.md).

## Aufgabe 2: Embedding lokal prüfen

- [x] Das vorhandene `text-embedding-nomic-embed-text-v1.5` laden.
- [x] Über einen autorisierten lokalen Client drei harmlose deutsche Testsätze einbetten; drei Vektoren mit je 768 Dimensionen wurden geliefert.
- [x] Mit einer kleinen, vom Team selbst geschriebenen Sammlung prüfen, ob ähnliche Projekttexte näher zueinander liegen als ein themenfremder Text. Kosinusähnlichkeit: 0,8036 gegenüber 0,5959.
- [x] Modell wieder entladen; ein früherer Ladevorgang dauerte laut LM Studio 2,96 s. Ein reproduzierter Kaltstart-Benchmark steht noch aus.

**Abnahme:** Lokale API liefert reproduzierbar gültige Vektoren; ein einfacher semantischer Abruf findet den erwarteten Text. Fehlt ein autorisierter Zugang, wird nur dieser Test pausiert, ohne die Tokenpflicht abzuschalten. Qwen3-Embedding-0.6B wird erst bei Qualitätsbedarf als Vergleich hinzugezogen.

**Ausführungsstand:** Der autorisierte Embedding-Funktionstest ist bestanden. Der temporär bereitgestellte Zugang wurde nicht gespeichert; wegen Offenlegung im Chat muss er widerrufen werden. Qualitäts- und Kaltstart-Benchmarks bleiben offen.

## Aufgabe 3: Bild-Eingang der konkreten Qwen-Datei prüfen

- [ ] Die Q4-Datei `gamedev-qwen3.8-v2-rea@q4_k_m` und den vorhandenen passenden Projektor in der Runtime zuordnen.
- [ ] Mit einem künstlich erzeugten Testbild eine konkrete Bildfrage stellen, beispielsweise die Farbe eines großen Rechtecks.
- [ ] Prüfen, ob der Aufruf wirklich Bilddaten enthält und die Antwort ohne Bild anders ausfällt.
- [ ] Bei fehlender Projektorunterstützung die Qwen-Ableitung als textfähig markieren; erst danach eine offiziell passende multimodale Paketierung erwägen.

**Abnahme:** Bildinhalt wird bei wiederholten Tests korrekt erkannt; Modell-, Projektor- und Runtime-Version stehen im Protokoll. Ein Modellname oder vorhandenes `mmproj` allein reicht nicht als Nachweis.

## Aufgabe 4: Spracheingabe und optionale Ausgabe auswählen

- [ ] Vorhandene lokale ASR- und TTS-Runtimes inventarisieren.
- [ ] Für Spracheingabe einen Kandidaten mit deutschem, freigegebenem Testaudio prüfen; Transkript, Latenz und Offline-Verhalten erfassen.
- [ ] Sprachausgabe separat bewerten, sobald die Voice-Oberfläche spezifiziert ist.

**Abnahme:** Spracheingabe liefert ein verständliches deutsches Transkript ohne externe Anfrage. Embeddings werden ausschließlich für Suche und Memory verwendet. Ein zusätzlicher Download erfolgt erst nach Inventur und Kandidatenentscheidung.

## Aufgabe 5: Hauptmodell und Laufzeit vergleichen

- [ ] Q4- und Q6-Qwen-Ableitung mit identischen kurzen Text-, Werkzeug- und gegebenenfalls Bildaufgaben testen.
- [ ] Pro Konfiguration Kaltstart, erste Antwort, Tokenrate, maximale VRAM-Belegung, RAM und Entladen nach Leerlauf messen; mehrere Läufe statt eines Einzelwerts.
- [ ] LM Studio mit vorhandener llama.cpp-Engine als Ausgangspunkt verwenden. Ollama nur vergleichen, wenn Aufgabe 1 einen stabilen Modellpfad ergibt.
- [ ] Prüfen, ob Thinking-Ausgabe für einfache Nutzerfragen begrenzt oder unterdrückt werden kann, ohne die Antwortqualität unzulässig zu senken.

**Abnahme:** Eine Vergleichstabelle begründet die primäre Runtime und Modellvariante. Zielgrößen werden aus den Messungen festgelegt; es gibt derzeit keine belastbaren Grenzwerte.

## Aufgabe 6: Installationsmanifest und nächster Softwareblock

- [ ] Für jede gewählte Funktion Modellrolle, Quelle, Version, Dateiformat, Größe, Prüfsumme, Lizenz, Hardwarebedarf und Offline-Fähigkeit erfassen.
- [ ] Wiederverwendung vorhandener Installationen, Downloads nach Zustimmung, Fehlerbehandlung und Entfernen AIDO-eigener Dateien spezifizieren.
- [ ] Einen separaten Implementierungsplan für den ersten Windows-Prototyp erstellen: Textdialog → Permission Gateway → eine freigegebene Desktop-Aktion → Memory-Abruf. Bild und Sprache folgen nach den Modellprüfungen.

**Abnahme:** Der Installer kann später genau erklären, was vorhanden ist, was zusätzlich gebraucht wird und welche Datenverarbeitung lokal bleibt. Erst dann beginnt die Implementierung des AIDO-Kerns.

## Reihenfolge und Ergebnis

Aufgaben 1 bis 3 können als Bestands- und Funktionsprüfung nacheinander abgeschlossen werden. Aufgabe 4 hängt von der Inventur lokaler Audio-Runtimes ab. Aufgabe 5 verwendet die Ergebnisse von 1 und 3. Aufgabe 6 ist die Entscheidungsvorlage für die erste AIDO-Implementierung. Jeder Prüfschritt endet mit beobachtetem Ergebnis oder ausdrücklich dokumentiertem Hindernis.
