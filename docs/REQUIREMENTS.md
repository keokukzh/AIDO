# Anforderungen

Status aller Punkte: **geplant**, sofern nicht ausdrücklich anders angegeben. Die Prioritäten ordnen die Entwicklung; sie belegen keine Umsetzung.

## Muss für einen ersten nutzbaren Prototyp

| ID | Anforderung |
| --- | --- |
| R-01 | Windows 11 als Zielplattform; sichtbarer, animierter Desktop-Charakter mit Textdialog. |
| R-02 | Hardware und vorhandene lokale Modelllaufzeiten erkennen; geeignete lokale Konfiguration anbieten. |
| R-03 | Betriebsmodus verständlich wählen und speichern: Private Offline, Local Connected oder Hybrid AI. |
| R-04 | Standard Access und optional Full Access anbieten; Rechte und automatische Ausführung getrennt konfigurieren. |
| R-05 | Jede folgenreiche Datei-, App- oder Systemänderung vor Ausführung mit Ziel, Wirkung und Freigabe anzeigen. |
| R-06 | Installierte Apps erkennen und pro App nur nach Zustimmung Fähigkeiten prüfen und aktivieren. |
| R-07 | Adapter bevorzugen, die eine dokumentierte API oder CLI nutzen; UI-Automation als Fallback prüfen. |
| R-08 | Aktionen, Freigaben, Fehler und Datenzugriffe lokal nachvollziehbar protokollieren. |
| R-09 | Dauerhaftes lokales Memory mit Quelle, Zeit, Gültigkeit, Suche und Löschmöglichkeit aufbauen. |
| R-10 | Setup-Schritte wiederaufnehmen; bestehende Programme und Modelle nicht ungefragt verändern. |
| R-11 | Für semantische Memory-Suche ein dediziertes lokales Text-Embedding-Modell mitinstallieren oder eine vorhandene kompatible Installation nach Prüfung wiederverwenden. |
| R-12 | Für Spracheingabe ein gesondertes lokales Spracherkennungsmodell samt Mikrofonfreigabe bereitstellen; Embeddings ersetzen keine Transkription. |
| R-13 | Bild-Eingang des gewählten Hauptmodells mit der tatsächlich eingesetzten Runtime und Modelldatei prüfen; Modellname allein gilt nicht als Nachweis. |
| R-14 | Vor Downloads vorhandene Inferenzlaufzeiten, Modellformate und Versionen erkennen; eine passende Runtime auswählen und unnötige Parallelinstallationen vermeiden. |

## Spätere Ausbaustufen

Sprachausgabe, proaktive Vorschläge, geplante Hintergrundaufgaben, Projektwissen, spezialisierte interne Agenten und isoliert erzeugte App-Adapter gehören zur Vision. Sie werden erst nach den Grundlagen für Berechtigungen und überprüfbare Ausführung integriert.

## Datenschutz und Sicherheit

Private Offline darf keine externen KI-Anfragen, Telemetrie, Websuche oder externen App-Verkehr durch AIDO auslösen. Local Connected hält Modell und Memory lokal, erlaubt einzelne freigegebene Netzwerkaktionen. Hybrid AI darf Cloud-KI nur nach expliziter Wahl und verständlicher Anzeige der zu übertragenden Daten verwenden. Für alle Modi sind Datenschutzprüfungen nötig.

Screenshots, Mikrofon, Dokumente, Chatverläufe und Projektordner erhalten gesonderte Freigaben. AIDO speichert Zugangsdaten nicht im Klartext im Memory. Ein privilegierter Broker darf nur eng definierte, bestätigte Aktionen ausführen. Windows-Schutzmechanismen werden nicht umgangen.

## Integrationsregeln

App-Erkennung, App-Bedienung und Kontozugriff sind verschiedene Fähigkeiten. Eine gefundene App bedeutet keinen Zugriff auf deren interne Daten. Für jede Fähigkeit werden Adapterversion, erforderliche Rechte, letzter erfolgreicher Test und Fehlerstatus erfasst. Neue Adapter werden nur in einer geeigneten isolierten Umgebung geprüft und erst danach freigegeben. Auf Windows 11 Home ist eine andere Isolation als Windows Sandbox vorzusehen, falls Sandbox nicht verfügbar ist.
