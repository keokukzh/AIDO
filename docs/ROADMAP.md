# Entwicklungsroadmap

AIDO soll sich per Windows-Installer einrichten lassen, als MONO auf dem Desktop erwachen und mit freigegebenen Projekten und Werkzeugen arbeiten. Bei Aktionen zeigt MONO das konkrete Ziel und die Wirkung. Der Nutzer kann zustimmen, ablehnen oder den Vorschlag ändern. Der [Projektstatus](PROJECT_STATUS.md) hält fest, welche Abläufe im installierten Build geprüft sind.

## 1. Installierten Desktop-Meilenstein abnehmen

Auf dem Referenz-PC AIDO 0.1.2 installieren und unterbrochenes Setup, Upgrade, Projektauswahl und Werkzeugstatus prüfen. REA im Werkzeugkatalog mit einer ausgewählten Windows-Programmdatei testen. Für «Öffne mein Projekt AIDO» muss Antigravity exakt `C:\Users\keoku\Desktop\AIDO` öffnen. Ablehnen, Ändern, abgelaufene und wiederverwendete Freigaben sowie das lokale Protokoll prüfen. Blender dient als Regression für den Programmstart. Im Werkzeugkatalog den letzten echten Test je Fähigkeit nachvollziehbar anzeigen.

Qwen in WSL2 auf dem Referenz-PC bis zur Bereitschaft bringen, zwei zusammenhängende Antworten prüfen und die VRAM-Freigabe nach Leerlauf messen. Den Windows-Modus als auswählbares Profil prüfen. AIDO-eigene Prozesse beim Beenden über den Infobereich stoppen und parallel prüfen, dass Hermes erreichbar bleibt.

## 2. Sprache, Screenshot und MONO-Bedienung

Grösse und Position speichern. Gedrückthalten nimmt Audio für lokales Whisper auf. Mittelklick hängt einen sichtbar markierten Screenshot an die nächste Nachricht an. Aufnehmen, Arbeiten, Freigabe und Fehler erhalten unterscheidbare Zustände. Abnahme mit tatsächlich gesprochener Eingabe, sichtbarem Anhang und nachvollziehbarer Vision-Antwort. Die Modellpfade und Messungen stehen in der [Laufzeitübersicht](MODEL_RUNTIME_INVENTORY.md).

## 3. Gedächtnis, proaktive Hilfe und Aufgaben

SQLite/FTS5 an den nativen Chat und Projektkatalog anbinden. Sitzungsverlauf, verdichtete Erinnerungen und Projektwissen mit Herkunft, Speicherlimit, Korrektur, Export und Löschung verwalten. Embeddings anhand konkreter Projektfragen gegen die Volltextsuche prüfen. Proaktive Vorschläge aus freigegebenen Projekten und lokalen Systemsignalen ableiten. Ruhezeiten, Vollbild, Stummschalten und Rückmeldungen beachten. Den Wochenbericht als ersten planbaren Skill mit sichtbarem Start, Pause, Abbruch und Ergebnis testen.

## 4. Aktionen, Skills und Integrationen

Geprüfte Programmstarts und einzelne App-Funktionen über fest definierte Aktionen anbinden. Blender-Funktionen im Programm über einen eigenen Adapter prüfen. Projektbezogene Subagenten arbeiten mit ausgewähltem Ordner, Auftrag und begrenztem Werkzeugset. REA bleibt eine lesende Dateianalyse. Fast Cleanup zuerst auf AIDOs Cache begrenzen. Deep Cleanup erhält Zielpfade, Vorschau und eigene Freigabe. Für privilegierte Aktionen einen echten Windows-UAC-Ablauf und für gespeicherte Zugangsdaten Windows-DPAPI integrieren und prüfen.

GitHub, Gmail, Outlook und Google Drive einzeln aktivierbar und widerrufbar machen. Android und die sichere Smartphone-Fernbedienung als eigene Abläufe abnehmen. Docker bleibt optional und braucht eine getrennte Prüfung inklusive GPU-Zugriff. Modelltraining folgt erst bei einer reproduzierbaren Qualitätslücke.

## 5. Windows-Produkt veröffentlichen

Installer auf einem sauberen Windows-11-System ohne Entwicklungswerkzeuge und auf einem schwächeren Rechner testen. Signierten Installer, Updates und Deinstallation abnehmen. Projekte, Modelle und lokale Nutzerdaten bei Upgrade und Deinstallation erhalten und die Wiederherstellung prüfen. Offlinebetrieb und niedrige Hardware-Ausstattung in die reproduzierbare Release-Prüfung aufnehmen.

## Bisherige Grundlagen

Die Phasen 0–13 lieferten Rust-Kernmodule, CLI-Demos und die Browser-Oberfläche. Dazu gehören Freigaben, Projektkatalog, Gedächtnis, Bereinigung, Aufgaben, Spezialisten, REA-Prototyp, Android-ADB und Agent-Remote-Entwürfe. Die Browser-Vorschau in Phase 14 ergänzt MONO. Der [Masterplan](MASTERPLAN.md) beschreibt die Produktidee und der [Projektstatus](PROJECT_STATUS.md) die native Abnahme.
