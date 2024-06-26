use std::path::PathBuf;
use structopt::StructOpt;

mod download;
mod install;
mod package;
mod recipe;
mod rime_levers;

use download::下載配方包;
use install::安裝配方;
use package::配方包;
use recipe::配方名片;
use rime_levers::{製備輸入法固件, 設置引擎啓動參數, 配置補丁};

#[derive(Debug, StructOpt)]
#[structopt(about = "Rime 配方管理器")]
enum 子命令 {
    /// 下載配方包
    Download { recipes: Vec<String> },
    /// 安裝配方
    Install { recipes: Vec<String> },
    /// 配置補丁
    Patch {
        /// 目標配置
        config: String,
        /// 紐
        key: String,
        /// 值
        value: String,
    },
    /// 新建配方
    NewRecipe {
        /// 配方名字
        name: Option<String>,
    },
    /// 構建輸入法固件
    Build,
    /// 部署輸入法固件到目標位置
    Deploy,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let 命令行參數 = 子命令::from_args();
    log::debug!("參數: {:?}", 命令行參數);
    match 命令行參數 {
        子命令::Download { ref recipes } => {
            for rx in recipes {
                下載配方包(配方包::from(rx.as_str()).倉庫);
            }
        }
        子命令::Install { ref recipes } => {
            for rx in recipes {
                安裝配方(配方名片::from(rx.as_str()));
            }
        }
        子命令::Patch {
            ref config,
            ref key,
            ref value,
        } => {
            let 還不知道怎麼傳過來 = PathBuf::from(".");
            設置引擎啓動參數(&還不知道怎麼傳過來)?;
            配置補丁(config, key, value)?;
        }
        子命令::Build => {
            let 還不知道怎麼傳過來 = PathBuf::from(".");
            設置引擎啓動參數(&還不知道怎麼傳過來)?;
            製備輸入法固件()?;
        }
        _ => todo!("還沒做呢"),
    }

    Ok(())
}
