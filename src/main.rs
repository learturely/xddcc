#![feature(let_chains)]
#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(iter_next_chunk)]
#![feature(iter_array_chunks)]

mod live;
mod protocol;
mod room;
mod tools;

use crate::{live::Live, room::Room, tools::PairVec};
use clap::Parser;
use cxsign::store::{tables::AccountTable, DataBase};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "获取直播信息。")]
pub struct Args {
    /// 获取特定账号下节课的直播信息，格式为以半角逗号隔开的字符串。
    #[arg(short, long)]
    pub accounts: Option<String>,
    /// 覆盖默认行为至获取当前课的直播信息。
    #[arg(short, long)]
    pub this: bool,
    /// 通过 `device_code` 获取直播信息。
    #[arg(short, long)]
    pub device_code: Option<String>,
    /// 导出文件路径。可选提供。
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// 列出所有设备码。
    #[arg(short, long)]
    pub list: bool,
    // /// 网页播放器地址。
    // #[arg(short, long)]
    // pub web: bool,
}
fn main() {
    let Args {
        accounts,
        this,
        device_code,
        output,
        list,
        // web,
    } = <Args as clap::Parser>::parse();
    let db = DataBase::default();
    let table = db.add_table::<AccountTable>();
    if list {
        if device_code.is_some() {
            eprintln!("多余的参数: `-d, --device-code`.")
        }
        if this {
            eprintln!("多余的参数: `-t, --this`.")
        }
        // if web {
        //     eprintln!("多余的参数: `-w, --web`.")
        // }
        let sessions = if let Some(accounts) = accounts {
            table.get_sessions_by_accounts_str(&accounts)
        } else {
            table.get_sessions()
        };
        if sessions.len() == 0 {
            eprintln!("请至少登录一个账号！");
        }
        let rooms = tools::map_sort_by_key(Room::get_all_rooms(sessions.values()));
        tools::out(&PairVec::new(rooms), output)
    } else if let Some(device_code) = device_code {
        if accounts.is_some() {
            eprintln!("多余的参数: `-a, --accounts`.")
        }
        if this {
            eprintln!("多余的参数: `-t, --this`.")
        }
        let sessions = table.get_sessions();
        if sessions.len() == 0 {
            eprintln!("未有登录的账号！");
        }
        for session in sessions.values() {
            tools::out(
                &tools::get_live_video_path(session, &device_code),
                output.clone(),
            );
            break;
        }
    } else {
        let sessions = if let Some(accounts) = accounts {
            table.get_sessions_by_accounts_str(&accounts)
        } else {
            table.get_sessions()
        };
        if sessions.len() == 0 {
            eprintln!("未有登录的账号！");
        }
        tools::out(
            &PairVec::new(tools::map_sort_by_key(Live::get_lives_now(
                sessions.values(),
                this,
            ))),
            output.clone(),
        );
    }
}
