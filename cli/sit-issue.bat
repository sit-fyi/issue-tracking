@echo off
set script=%~dp0%\sit-issue.ps1
PowerShell -NoProfile -ExecutionPolicy Bypass -Command "& '%script%' %*";
