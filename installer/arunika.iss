; Arunika installer (Inno Setup 6.5.2+, for the PNG wizard images).
;
; Releases ship installer/arunika.nsi (NSIS), which 0.1.x installs came from; CI builds this one too as a check.
;
; Build after `cargo build -p poni --release`:
;   iscc installer\arunika.iss                      -> dist\Arunika-Setup-0.2.6.exe
;   iscc /DAppVersion=0.2.0 installer\arunika.iss   (another version)
;
; Per-user install: no admin rights and no UAC prompt. The app goes to
; %LOCALAPPDATA%\Programs\Arunika; settings and data stay in %APPDATA%\Arunika and
; survive uninstalling (only the cache in %LOCALAPPDATA%\Arunika is removed).

#ifndef AppVersion
  #define AppVersion "0.2.6"
#endif
#ifndef ExeSource
  #define ExeSource "..\target\release\arunika.exe"
#endif
#define AppName "Arunika"
; Folder, data and Run value names are "Arunika" (the app uses them).
#define AppDir "Arunika"
#define AppPublisher "Arunika Labs"
#define AppExe "arunika.exe"
; The island window's class (crates/poni/src/window.rs), used to ask a running copy to quit.
#define IslandClass "Arunika.Island"

[Setup]
; Never change the AppId: upgrades and the uninstaller find the install by it.
AppId={{6D9489C4-A934-475C-9F3F-E293B72D4775}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppCopyright=Copyright (C) 2026 {#AppPublisher}
VersionInfoVersion={#AppVersion}
VersionInfoCompany={#AppPublisher}
VersionInfoDescription={#AppName} Setup
VersionInfoProductName={#AppName}
PrivilegesRequired=lowest
DefaultDirName={localappdata}\Programs\{#AppDir}
DisableDirPage=yes
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir=..\dist
OutputBaseFilename=Arunika-Setup-{#AppVersion}
SetupIconFile=..\crates\poni\res\arunika.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
WizardStyle=modern
; Oyen hanging from the island on the finish page, Arunika's logo in the header (both from tools/make_brand.py).
; Setup picks the file closest to its image area: 100%, 150% and 200% scaling.
WizardImageFile=art\wizard-202x386.png,art\wizard-336x643.png,art\wizard-430x824.png
WizardSmallImageFile=art\wizard-small-58.png,art\wizard-small-77.png,art\wizard-small-97.png,art\wizard-small-116.png,art\wizard-small-124.png,art\wizard-small-143.png,art\wizard-small-159.png
Compression=lzma2/max
SolidCompression=yes
; A running copy is asked to quit in PrepareToInstall; Restart Manager is the safety net.
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "autostart"; Description: "Start with Windows"; GroupDescription: "Startup:"

[Files]
Source: "{#ExeSource}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD-PARTY-NOTICES.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"; Comment: "A living notch for Windows"

[Registry]
; Same value the app writes from its "Start with Windows" menu item (autostart.rs): the quoted exe path.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#AppDir}"; ValueData: """{app}\{#AppExe}"""; Flags: uninsdeletevalue; Tasks: autostart

[Run]
Filename: "{app}\{#AppExe}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Make sure no copy is still running before its files are removed.
Filename: "{sys}\taskkill.exe"; Parameters: "/IM {#AppExe} /F"; Flags: runhidden waituntilterminated; RunOnceId: "StopArunika"

[UninstallDelete]
; Cache and log only (lyrics, arunika.log); settings and data in %APPDATA%\Arunika are kept.
Type: filesandordirs; Name: "{localappdata}\{#AppDir}"

[Code]
const
  RunKey = 'Software\Microsoft\Windows\CurrentVersion\Run';
  WM_CLOSE = $0010;

{ Ask a running Arunika to quit (it saves its data on the way out); force it if it doesn't within 3 s. }
procedure StopArunika();
var
  I, ResultCode: Integer;
begin
  if FindWindowByClassName('{#IslandClass}') = 0 then
    Exit;
  PostMessage(FindWindowByClassName('{#IslandClass}'), WM_CLOSE, 0, 0);
  for I := 1 to 30 do
  begin
    if FindWindowByClassName('{#IslandClass}') = 0 then
      Exit;
    Sleep(100);
  end;
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/IM {#AppExe} /F', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  StopArunika();
  Result := '';
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  { Unticking "Start with Windows" on a reinstall turns it off, even if the app's menu turned it on. }
  if (CurStep = ssPostInstall) and not WizardIsTaskSelected('autostart') then
    RegDeleteValue(HKEY_CURRENT_USER, RunKey, '{#AppDir}');
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  { Runs before [UninstallRun], so a running copy gets the chance to save and quit cleanly. }
  if CurUninstallStep = usUninstall then
    StopArunika();
  { The app may have written the Run value itself; never leave it pointing at a removed exe. }
  if CurUninstallStep = usPostUninstall then
    RegDeleteValue(HKEY_CURRENT_USER, RunKey, '{#AppDir}');
end;
