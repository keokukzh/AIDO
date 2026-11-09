# Entscheidungen und offene Fragen

Die Einträge geben die Entscheidungen aus den AIDO-Interviews vom 9. November 2026 wieder. Technische Vorschläge aus den Assistentenantworten sind als Kandidaten gekennzeichnet.

## Produktentscheidungen

| Thema | Entscheidung |
| --- | --- |
| Zielgruppe | Normale PC-Nutzer ohne technische Kenntnisse. |
| Plattform | Windows 11. |
| KI | Lokal bevorzugt, wenn Hardware geeignet ist; Cloud-Nutzung bleibt wählbar. |
| Betriebsarten | Private Offline, Local Connected und Hybrid AI als geplante Datenschutzmodi. |
| Zugriffsprofile | Standard Access und optional Full Access; Funktionen und Folgen im Installer erklären. |
| Freigaben | Umfassender Zugriff ist möglich, aber vor jeder relevanten Änderung nachfragen. |
| Sensible Quellen | Lokale Chats, Dokumente und Projekte nur nach Einzelzustimmung einbeziehen. |
| Proaktivität | Vorschläge, Aufgabenplanung und länger laufende Automatisierungen sind gewünscht; neue Ziele zur Bestätigung vorschlagen. |
| App-Anbindung | Apps erkennen und je Nutzer auswählbar verbinden; vorhandenen Adapter zuerst prüfen, visuelle Bedienung als Fallback. |
| Spezialisten | Apps können über zulässige Schnittstellen als spezialisierte Werkzeuge dienen. |
| Adapterbau | Neue Adapter dürfen geplant und in Isolation getestet werden; Aktivierung folgt Prüfung und Freigabe. |
| Installation | Zwei wählbare Installationswege; gründliche Einrichtung und ehrliche Zeitangabe sind wichtiger als kurze Dauer. |
| Entwicklungsumgebung | Antigravity ist nur Teil der persönlichen Entwicklungsumgebung, keine Produktpflicht. |
| Produktumfang | Desktop-Assistent, System-/App-Hilfe und Projektunterstützung gehören zur Vision; MVP schrittweise. |
| Geschäftsmodell | Offen. |

## Architekturvorschläge zur Prüfung

Tauri 2, Rust/TypeScript, Three.js, Qwen3.8-27B, Hermes, SQLite/FTS5, Mem0, Windows UI Automation und genannte Adapterprojekte sind Kandidaten aus der Planung. Ihre technische Eignung, Lizenzen und Wartbarkeit sind noch zu prüfen. Docker und WSL2 sind für Endnutzer nicht als Pflicht vorgesehen.

## Offene Entscheidungen

Konkretes MVP-Szenario, Modellwahl pro Hardwareklasse, Sicherheitsgrenze des Full-Access-Brokers, Datenaufbewahrung, Updatekanal, Lizenz und Geschäftsmodell sind offen. Der Docker-Anhang ist in der verfügbaren Referenz nicht enthalten; Docker-spezifische Beschlüsse folgen erst nach Sichtung.

Die Formulierung „100-mal schneller und besser“ ist eine Vision, keine bestätigte Messung oder Garantie.
