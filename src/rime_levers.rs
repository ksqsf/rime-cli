use anyhow::{anyhow, bail};
use rime::{
    rime_api_call, rime_module_call, rime_struct_new, RimeConfig, RimeLeversApi, RimeTraits,
};
use std::ffi::CString;
use std::path::PathBuf;

pub fn 設置引擎啓動參數(工作場地: &PathBuf) -> anyhow::Result<()> {
    log::debug!("設置引擎啓動參數. 工作場地: {}", 工作場地.display());
    std::fs::create_dir_all(工作場地)?;
    let 場地〇 = CString::new(工作場地.to_str().ok_or(anyhow!("路徑編碼轉換錯誤"))?)?;
    let 品名〇 = CString::new(env!("CARGO_PKG_NAME"))?;
    let 版本〇 = CString::new(env!("CARGO_PKG_VERSION"))?;
    let mut 啓動參數: RimeTraits = rime_struct_new!();
    啓動參數.data_size = std::mem::size_of::<RimeTraits>() as std::ffi::c_int;
    啓動參數.shared_data_dir = 場地〇.as_ptr();
    啓動參數.user_data_dir = 場地〇.as_ptr();
    啓動參數.distribution_name = 品名〇.as_ptr();
    啓動參數.distribution_code_name = 品名〇.as_ptr();
    啓動參數.distribution_version = 版本〇.as_ptr();
    rime_api_call!(setup, &mut 啓動參數);
    Ok(())
}

pub fn 製備輸入法固件() -> anyhow::Result<()> {
    log::debug!("製備輸入法固件");
    rime_api_call!(deployer_initialize, std::ptr::null_mut());
    rime_api_call!(deploy);
    rime_api_call!(finalize);
    Ok(())
}

pub fn 配置補丁(目標配置: &str, 紐: &str, 值: &str) -> anyhow::Result<()> {
    log::debug!("配置補丁: {目標配置}:/{紐} = {值}");

    let 目標配置〇 = CString::new(目標配置)?;
    let 紐〇 = CString::new(紐)?;
    let 值〇 = CString::new(值)?;

    let mut 值解析爲節點樹: RimeConfig = rime_struct_new!();
    if rime_api_call!(config_load_string, &mut 值解析爲節點樹, 值〇.as_ptr()) == 0 {
        bail!("無效的 YAML 值: {}", 值);
    }

    let levers_模塊名〇 = CString::new("levers")?;
    let levers = rime_api_call!(find_module, levers_模塊名〇.as_ptr());
    if levers.is_null() {
        bail!("沒有 levers 模塊");
    }

    let 配置工具名稱〇 = CString::new("rime-cli")?;
    let 自定義配置 = rime_module_call!(
        levers => RimeLeversApi,
        custom_settings_init,
        目標配置〇.as_ptr(),
        配置工具名稱〇.as_ptr()
    );

    // 可能已有自定義配置, 先加載
    rime_module_call!(levers => RimeLeversApi, load_settings, 自定義配置);
    // 生成補丁
    if rime_module_call!(
        levers => RimeLeversApi,
        customize_item,
        自定義配置,
        紐〇.as_ptr(),
        &mut 值解析爲節點樹
    ) != 0
    {
        rime_module_call!(levers => RimeLeversApi, save_settings, 自定義配置);
        log::info!("補丁打好了. {目標配置}:/{紐}");
    }

    rime_module_call!(levers => RimeLeversApi, custom_settings_destroy, 自定義配置);
    rime_api_call!(config_close, &mut 值解析爲節點樹);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use claims::assert_ok;
    use lazy_static::lazy_static;
    use std::fs::{read_to_string, write};
    use std::io::ErrorKind;
    use std::sync::Once;

    lazy_static! {
        static ref 測試場地: PathBuf = std::env::temp_dir().join("rime_levers_tests");
    }

    static 預備本場測試: Once = Once::new();

    fn 預備() {
        預備本場測試.call_once(|| {
            match std::fs::remove_dir_all(&*測試場地) {
                Ok(()) => {}
                Err(e) if e.kind() == ErrorKind::NotFound => {}
                e @ _ => assert_ok!(e),
            }
            assert_ok!(設置引擎啓動參數(&測試場地));
        })
    }

    #[test]
    fn 測試配置補丁_全局配置() {
        預備();
        assert_ok!(配置補丁("test_default", "menu/page_size", "5"));

        let 結果文件 = 測試場地.join("test_default.custom.yaml");
        let 補丁文件內容 = assert_ok!(read_to_string(&結果文件));
        assert!(補丁文件內容.contains(
            r#"
patch:
  "menu/page_size": 5"#
        ));
    }

    #[test]
    fn 測試配置補丁_輸入方案() {
        預備();
        assert_ok!(配置補丁("test_ohmyrime.schema", "menu/page_size", "9"));

        let 結果文件 = 測試場地.join("test_ohmyrime.custom.yaml");
        let 補丁文件內容 = assert_ok!(read_to_string(&結果文件));
        assert!(補丁文件內容.contains(
            r#"
patch:
  "menu/page_size": 9"#
        ));
    }

    #[test]
    fn 測試配置補丁_列表值() {
        預備();
        assert_ok!(配置補丁(
            "test_patch_list",
            "starcraft/races",
            r#"[protoss, terran, zerg]"#
        ));

        let 結果文件 = 測試場地.join("test_patch_list.custom.yaml");
        let 補丁文件內容 = assert_ok!(read_to_string(&結果文件));
        println!("補丁文件內容: {}", 補丁文件內容);
        assert!(補丁文件內容.contains(
            r#"
patch:
  "starcraft/races":
    - protoss
    - terran
    - zerg"#
        ));
    }

    #[test]
    fn 測試配置補丁_字典值() {
        預備();
        assert_ok!(配置補丁(
            "test_patch_map",
            "starcraft/workers",
            r#"{protoss: probe, terran: scv, zerg: drone}"#
        ));

        let 結果文件 = 測試場地.join("test_patch_map.custom.yaml");
        let 補丁文件內容 = assert_ok!(read_to_string(&結果文件));
        assert!(補丁文件內容.contains(
            r#"
patch:
  "starcraft/workers":
    protoss: probe
    terran: scv
    zerg: drone"#
        ));
    }

    // 按: 測試配置補丁無需任何組件, 且與本項測試訪問的文件不衝突,
    // 因此各個測試用例可以併發執行.
    #[test]
    fn 測試製備輸入法固件() {
        預備();
        assert_ok!(write(
            測試場地.join("default.yaml"),
            r#"
schema_list:
  - schema: ohmyrime
"#,
        ));
        assert_ok!(write(
            測試場地.join("ohmyrime.schema.yaml"),
            r#"
schema:
  schema_id: ohmyrime
"#,
        ));

        assert_ok!(製備輸入法固件());

        assert!(測試場地.join("installation.yaml").exists());
        assert!(測試場地.join("user.yaml").exists());
        let 整備區 = 測試場地.join("build");
        let 默認配置文件 = 整備區.join("default.yaml");
        let 默認配置內容 = assert_ok!(read_to_string(&默認配置文件));
        assert!(默認配置內容.contains(
            r#"
schema_list:
  - schema: ohmyrime"#
        ));
        let 輸入方案文件 = 整備區.join("ohmyrime.schema.yaml");
        let 輸入方案內容 = assert_ok!(read_to_string(&輸入方案文件));
        assert!(輸入方案內容.contains(
            r#"
schema:
  schema_id: ohmyrime"#
        ));
    }
}
