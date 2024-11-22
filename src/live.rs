// Copyright (C) 2024 learturely <learturely@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{
    room::Room,
    tools::{json_parsing_error_handler, VideoPath},
    ProgressState, ProgressTracker, ProgressTrackerHolder,
};
use cxlib_user::Session;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Live {
    place: String,
    id: i64,
    #[serde(rename = "weekDay")]
    week_day: u32,
    jie: i32,
}
impl Live {
    pub fn get_id(&self) -> i64 {
        self.id
    }
    pub fn get_week_day(&self) -> u32 {
        self.week_day
    }
    pub fn get_jie(&self) -> i32 {
        self.jie
    }
    pub fn get_lives(
        session: &Session,
        week: i64,
        term_year: i32,
        term: i32,
    ) -> Result<HashMap<String, i64>, Box<ureq::Error>> {
        let vec = crate::protocol::list_student_course_live_page(session, week, term_year, term)?
            .into_json::<Vec<Live>>()
            .unwrap_or_else(json_parsing_error_handler);
        let mut map = HashMap::new();
        for i in vec {
            map.insert(i.place, i.id);
        }
        Ok(map)
    }
    fn get_lives_by_time(
        session: &Session,
        term_year: i32,
        term: i32,
        week: i64,
        week_day: u32,
        jie: i32,
    ) -> Result<Option<Live>, Box<ureq::Error>> {
        let vec = crate::protocol::list_student_course_live_page(session, week, term_year, term)?
            .into_json::<Vec<Live>>()
            .unwrap_or_else(json_parsing_error_handler);
        let iter = vec
            .into_iter()
            .filter(|live| (live.get_week_day() == week_day) && (live.get_jie() >= jie));
        let mut vec = iter.collect::<Vec<_>>();
        vec.sort_by_key(|live| live.get_jie());
        Ok(vec.first().cloned())
    }
    pub fn get_lives_now<
        'a,
        Iter: Iterator<Item = &'a Session> + Clone,
        P: ProgressTracker + 'static,
    >(
        sessions: Iter,
        previous: bool,
        multi: &impl ProgressTrackerHolder<P>,
    ) -> HashMap<&'a str, (&'a str, Room, VideoPath)> {
        let sessions = sessions.collect::<Vec<_>>();
        let total = sessions.len() as u64;
        let data_time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());
        let mut term_year = 0;
        let mut term = 0;
        let mut week = 0;
        let mut first = true;
        let week_day = chrono::Datelike::weekday(&data_time).number_from_monday();
        // `Session` 的 `Hash` 实现不涉及内部可变的字段。
        #[allow(clippy::mutable_key_type)]
        let mut lives_map: HashMap<&Session, Live> = HashMap::new();
        let pb = multi.init(total, ProgressState::GetLiveIds);
        for session in sessions.clone() {
            if !pb.go_on() {
                debug!("list_rooms/get_all_live_id: break.");
                break;
            }
            if first {
                (term_year, term, week) = crate::tools::term_year_detail(session);
                first = false;
            }
            let jie = crate::tools::now_to_jie(previous);
            let live = Live::get_lives_by_time(session, term_year, term, week, week_day, jie);
            if let Ok(Some(live)) = live {
                lives_map.insert(session, live);
            }
            pb.inc(1)
        }
        pb.finish(ProgressState::GetLiveIds);
        multi.remove_progress(&pb);
        let mut lives = HashSet::new();
        for live in lives_map.values() {
            lives.insert(live.get_id());
        }
        let mut rooms = HashMap::new();
        let pb = multi.init(total, ProgressState::GetLiveUrls);
        pb.inc(0);
        if let Some(session) = sessions.clone().into_iter().next() {
            for live in lives {
                if !pb.go_on() {
                    debug!("list_rooms/id_to_rooms: break.");
                    break;
                }
                match Room::get_rooms(session, live) {
                    Ok(room) => {
                        if let Some(room) = room {
                            pb.inc(1);
                            let video_path = room.get_live_video_path(session);
                            pb.inc(1);
                            rooms.insert(live, (room, video_path));
                        } else {
                            pb.inc(2);
                        }
                    }
                    Err(e) => {
                        warn!("教室获取错误：{e}.");
                        pb.inc(2);
                    }
                }
            }
        }
        let mut results = HashMap::new();
        for (session, live) in lives_map {
            if let Some((room, video_path)) = rooms.get(&live.get_id()) {
                match video_path {
                    Ok(video_path) => {
                        results.insert(
                            session.get_uid(),
                            (session.get_stu_name(), room.clone(), video_path.clone()),
                        );
                    }
                    Err(e) => {
                        warn!("获取教室失败：{e}.")
                    }
                }
            }
        }
        pb.finish(ProgressState::GetLiveUrls);
        multi.remove_progress(&pb);
        results
    }
}
