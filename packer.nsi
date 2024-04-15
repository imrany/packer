; Example NSIS script for installing a Rust program

; Define the name and version of your program
!define APP_NAME "packer"
!define APP_VERSION "0.2.1"

; The name of the installer
Name "packer-0.2.1-setup"

; The file to write
OutFile "packer-0.2.1-setup.exe"

; Default installation directory
InstallDir "$PROGRAMFILES\${APP_NAME}"

; Request application privileges for installation
RequestExecutionLevel admin

; Registry key to check for directory (so if you install again, it will 
; overwrite the old one automatically)
InstallDirRegKey HKLM "Software\${APP_NAME}" "Install_Dir"

;--------------------------------

; Pages

Page components
Page directory
Page instfiles

UninstPage uninstConfirm
UninstPage instfiles

;--------------------------------

; Begin installer section
Section
    SetOutPath "$INSTDIR"

   ; Extract files from the .zip archive
    File "${APP_NAME}.zip"

    ; Unzip the archive to the installation directory
    DetailPrint "Extracting files..."
    ExecWait '"$PLUGINSDIR\unzip.exe" "$PLUGINSDIR\${APP_NAME}.zip" -d "$INSTDIR"'
    DetailPrint "Files extracted successfully."

    ; Write the installation path into the registry
    WriteRegStr HKLM "SOFTWARE\${APP_NAME}" "Install_Dir" "$INSTDIR"
    
    ; Write the uninstall keys for Windows
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" '"$INSTDIR\uninstall.exe"'
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "NoRepair" 1
    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; End installer section
SectionEnd

; Optional uninstaller section
Section "Uninstall"
    Delete "$INSTDIR\*.*" ; Delete all files in the installation directory
    RMDir "$INSTDIR" ; Remove the installation directory

    ; Remove registry keys
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
    DeleteRegKey HKLM "SOFTWARE\${APP_NAME}"

    ; Remove directories
    RMDir "$SMPROGRAMS\${APP_NAME}" 
    RMDir "$PROGRAMFILES\${APP_NAME}"
SectionEnd
