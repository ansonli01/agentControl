use std::path::PathBuf;

/// 应用清单（Application Manifest）必须随可执行文件一起嵌入，
/// 否则依赖 `comctl32` v6 的 `TaskDialogIndirect` 等函数无法解析，
/// 进程会以 STATUS_ENTRYPOINT_NOT_FOUND (0xC0000139) 启动失败。
///
/// `tauri-build` 默认把清单打进它自己的 Windows 资源里，而该资源只会
/// 链接到 `bin` 目标，`cargo test` 产生的 lib 测试可执行文件拿不到清单。
/// 因此这里让 tauri 只生成图标与版本信息，清单改由链接器统一嵌入，
/// 从而同时覆盖 bin 目标与测试可执行文件。
fn main() {
    let attributes = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());

    if let Err(error) = tauri_build::try_build(attributes) {
        panic!("failed to run tauri-build: {error:#}");
    }

    embed_manifest();
}

fn embed_manifest() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    if out_dir.is_empty() {
        return;
    }

    let manifest_path = PathBuf::from(&out_dir).join("agentcontrol.manifest");
    let manifest = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    type="win32"
    name="AgentControl"
    version="0.1.0.0"
    processorArchitecture="*"/>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"/>
    </dependentAssembly>
  </dependency>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">permonitorv2,permonitor</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>
"#;

    if std::fs::write(&manifest_path, manifest).is_err() {
        return;
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    // 注意：link.exe 会把带引号的路径当作相对路径处理，这里必须使用未加引号的绝对路径
    println!(
        "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
        manifest_path.display()
    );
}
