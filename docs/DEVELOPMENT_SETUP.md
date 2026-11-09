# Entwicklungsumgebung und Herkunft des Docker-Plans

**Stand: 9. November 2026.** Dieser Text ordnet den frühen Docker-Masterplan und den vorangehenden Produktgedanken ein. Beide sind Planungsquellen, keine Anweisungen zur Änderung des Rechners. Der ursprüngliche Plan zielte auf eine persönliche lokale KI- und Entwicklungsumgebung. Erst danach entstand die Idee eines allgemein nutzbaren AIDO-Produkts.

## Was aus dem frühen Plan passt

- Reproduzierbare, getrennte Entwicklungsumgebungen sind für Prototypen, Modellversuche und Adaptertests sinnvoll. Docker/WSL2 sind dafür optionale Werkzeuge auf dem Referenz-PC.
- Ein definierter Ort für Entwicklungsprojekte und klar benannte Daten-, Modell- und Backup-Pfade helfen gegen das beschriebene Projekt- und Pfadchaos.
- GPU-Tests, Compose-Validierung und kontrollierte Volumes sind gute Abnahmeschritte, falls Container tatsächlich eingesetzt werden.
- Die Idee eines schlummernden lokalen Hauptmodells passt zur AIDO-Vision. Ob Entladen und Wiederladen mit der gewählten Runtime praktikabel sind, muss gemessen werden.
- Der frühe Interaktionsentwurf nennt verschiebbaren Charakter, Klick für Chat, Gedrückthalten für Sprache, mittlere Maustaste für Screenshot als Prompt-Kontext und Rechtsklick für einfache Einstellungen. Das sind zu validierende Bedienkonzepte. Ein Screenshot darf erst nach erkennbarer Nutzeraktion und klarer Anzeige erfasst werden.
- Vorschläge in verständlicher Sprache mit „Ja“, „Nein“ und freier Ergänzung sind ein wichtiges Bedienprinzip. Dazu gehören Stopp, Vorschau und Protokoll für länger laufende Aufgaben.

## Was nicht als Produktvoraussetzung übernommen wird

Ein Endnutzer soll weder Docker, WSL2, Dev Containers, PostgreSQL, Redis, Jupyter noch n8n selbst einrichten müssen. Solche Komponenten kommen nur dort in Betracht, wo ein konkreter Nutzen nachgewiesen ist. Antigravity und andere persönliche IDEs gehören zur Entwicklungsumgebung, nicht zur Voraussetzung für AIDO. Eine universelle Admin-Vollmacht ersetzt keine Freigaben für folgenreiche Aktionen.

Der frühere Text nennt garantierte Leistungsaussagen wie „10x“ Dateisystemtempo oder GPU-Nutzung „ohne Leistungsverlust“. Dafür liegen hier keine Benchmarks vor. Die Aussagen werden nicht als Produktfakten übernommen. Ebenso ist eine vollständige Kenntnis aller Nutzerpfade oder Apps weder technisch noch datenschutzrechtlich automatisch gegeben.

## Gezielter Ist-Check am Referenz-PC

| Prüfung | Beobachtung |
| --- | --- |
| Docker Desktop | Client und Server meldeten Version 29.9.2; der Server war erreichbar. |
| GPU | `nvidia-smi` meldete eine GeForce RTX 5090 mit 32607 MiB und Treiberversion 617.42. |
| WSL2 | Ubuntu war als Standarddistribution vorhanden. |
| WSL-Konfiguration | `.wslconfig` enthielt 12 GB Arbeitsspeicher, 16 Prozessoren und 4 GB Swap. |
| Docker in Ubuntu | `docker info` schlug mit dem Hinweis auf fehlende WSL-Integration fehl. |
| Docker-Einstellung | `IntegratedWslDistros` war leer; die WSL-Engine war eingeschaltet. |
| Geplanter Sandbox-Ordner | Der im alten Plan genannte Ordner unter `/home/keokuk/docker-sandboxes` war nicht vorhanden. |
| Ubuntu-Sitzung | Der gezielte Test startete mit `/root` als Home; vor Pfadentscheidungen ist der gewünschte Linux-Benutzer zu prüfen. |

Dies ist eine punktuelle, lesende Prüfung. Es wurden weder Docker-Einstellungen geändert noch Container gestartet, Daten gelöscht, Projekte verschoben oder GPU-Passthrough getestet. Installierte Apps, Nutzungsfrequenz und persönliche Projektordner wurden nicht inventarisiert. Der alte Plan ist damit keine aktuelle Bestandsaufnahme.

## Vorschlag für die persönliche Entwicklungsumgebung

1. **Inventar und Schutzgrenzen:** Nur ausdrücklich gewählte Projektwurzeln, Modelle, Runtimes und Apps erfassen. Bestehende Daten sichern, bevor Pfade verändert werden. Kein automatisches 30-Tage-Screening persönlicher Chats oder Dateien ohne Einzelzustimmung.
2. **Pfade festlegen:** Einen kanonischen Quellcode-Ort in WSL2 wählen, sofern Linux-Tools benötigt werden; Windows-App und Installer nativ unter Windows entwickeln. Modelle, persistente Daten, temporäre Artefakte und Backups getrennt halten. Pfade als Konfiguration behandeln, nicht im Produkt fest verdrahten.
3. **Docker gezielt aktivieren:** Erst den Ubuntu-Benutzer und die Docker-Desktop-Integration klären. Danach `docker info` in Ubuntu und einen minimalen GPU-Test durchführen. Nur bei Erfolg eine kleine Dev-Container-Vorlage anlegen. Kein pauschales `docker system prune`, weil fremde Projekte betroffen sein können.
4. **AIDO-Kern zuerst:** Eine native Windows-Aktion mit verständlicher Freigabe, Protokoll und Abbruchmöglichkeit umsetzen. Container können parallele Modell- und Adapterexperimente isolieren, sind aber nicht Teil dieser Kernanforderung.
5. **Lokale KI messen:** Zwei Modell-/Runtime-Konfigurationen für Startzeit, Antwortzeit, Speicher und Idle-Entladung vergleichen. Qwen3.8-27B bleibt ein Kandidat. Memory zunächst mit lokaler strukturierter Speicherung und Suche testen; zusätzliche Frameworks nur bei messbarem Mehrwert.
6. **Produktisierung:** Erst nach dem Referenzprototyp Installer, Hardwareklassen und App-Auswahl auf unabhängigen Windows-11-Systemen prüfen.

## Offene Nachweise

Für die Docker-Entwicklungsumgebung fehlen ein erfolgreicher Ubuntu-Docker-Test, GPU-Passthrough, Compose-Dateien, Container-Healthchecks und Backup-/Restore-Tests. Für das AIDO-Produkt fehlen weiterhin Implementierung und Produkttests. Der Docker-Text wird deshalb als historischer Entwurf geführt, nicht als ausgeführtes Setup.
