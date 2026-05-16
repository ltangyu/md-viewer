; ══════════════════════════════════════════════════════════════
; MD Viewer — NSIS Installer Hooks
; Fixes Tauri bug #9803: missing Capabilities & RegisteredApplications
; Without these, the Windows "Open With" dialog loops infinitely.
; ══════════════════════════════════════════════════════════════

!macro CUSTOM_INSTALL
  ; ── Register Application ──
  WriteRegStr SHCTX "Software\Classes\Applications\md-viewer.exe" "FriendlyAppName" "MD Viewer"
  WriteRegStr SHCTX "Software\Classes\Applications\md-viewer.exe\shell\open\command" "" '"$INSTDIR\md-viewer.exe" "%1"'
  WriteRegStr SHCTX "Software\Classes\Applications\md-viewer.exe\SupportedTypes" ".md" ""
  WriteRegStr SHCTX "Software\Classes\Applications\md-viewer.exe\SupportedTypes" ".markdown" ""
  WriteRegStr SHCTX "Software\Classes\Applications\md-viewer.exe\SupportedTypes" ".mdx" ""

  ; ── Register Capabilities (THE FIX for Tauri #9803) ──
  WriteRegStr SHCTX "Software\MDViewer\Capabilities" "ApplicationName" "MD Viewer"
  WriteRegStr SHCTX "Software\MDViewer\Capabilities" "ApplicationDescription" "Lightweight Markdown Viewer"
  WriteRegStr SHCTX "Software\MDViewer\Capabilities\FileAssociations" ".md" "MDViewer.md"
  WriteRegStr SHCTX "Software\MDViewer\Capabilities\FileAssociations" ".markdown" "MDViewer.md"
  WriteRegStr SHCTX "Software\MDViewer\Capabilities\FileAssociations" ".mdx" "MDViewer.md"

  ; ── RegisteredApplications (required for Windows Default Apps) ──
  WriteRegStr SHCTX "Software\RegisteredApplications" "MDViewer" "Software\MDViewer\Capabilities"

  ; ── Add Content Type & PerceivedType ──
  WriteRegStr SHCTX "Software\Classes\.md" "Content Type" "text/markdown"
  WriteRegStr SHCTX "Software\Classes\.md" "PerceivedType" "text"
!macroend

!macro CUSTOM_UNINSTALL
  ; ── Clean up Capabilities ──
  DeleteRegKey SHCTX "Software\MDViewer"
  DeleteRegValue SHCTX "Software\RegisteredApplications" "MDViewer"

  ; ── Clean up Application registration ──
  DeleteRegKey SHCTX "Software\Classes\Applications\md-viewer.exe"

  ; ── Remove Content Type & PerceivedType ──
  DeleteRegValue SHCTX "Software\Classes\.md" "Content Type"
  DeleteRegValue SHCTX "Software\Classes\.md" "PerceivedType"
!macroend
