use std::collections::{HashMap, HashSet};

use cxsign::Session;
use serde::{Deserialize, Serialize};

use crate::{room::Room, tools::VideoPath};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Live {
    place: String,
    id: i64,
    #[serde(rename = "weekDay")]
    week_day: u32,
    jie: i32,
}
impl Live {
    pub fn get_id(&self) -> String {
        self.id.to_string()
    }
    pub fn get_week_day(&self) -> u32 {
        self.week_day
    }
    fn get_jie(&self) -> i32 {
        self.jie
    }
    pub fn get_lives(
        session: &Session,
        week: i64,
        term_year: i32,
        term: i32,
    ) -> Result<HashMap<String, i64>, ureq::Error> {
        let vec = crate::protocol::list_student_course_live_page(session, week, term_year, term)?
            .into_json::<Vec<Live>>()
            .unwrap();
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
    ) -> Result<Option<Live>, ureq::Error> {
        let vec = crate::protocol::list_student_course_live_page(session, week, term_year, term)?
            .into_json::<Vec<Live>>()
            .unwrap();
        let iter = vec
            .into_iter()
            .filter(|live| (live.get_week_day() == week_day) && (live.get_jie() >= jie));
        let mut vec = iter.collect::<Vec<_>>();
        vec.sort_by(|l1, l2| l1.get_jie().cmp(&l2.get_jie()));
        Ok(vec.first().cloned())
    }
    pub fn get_lives_now<'a, Iter: Iterator<Item = &'a Session> + Clone>(
        sessions: Iter,
        this: bool,
    ) -> HashMap<&'a str, (&'a str, Room, VideoPath)> {
        let sessions = sessions.collect::<Vec<_>>();
        let total = sessions.len() as u64;
        let data_time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());
        let mut term_year = 0;
        let mut term = 0;
        let mut week = 0;
        let mut first = true;
        let week_day = chrono::Datelike::weekday(&data_time).number_from_monday();
        let mut lives_map = HashMap::new();
        let sty = indicatif::ProgressStyle::with_template(
            "获取直播号：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap();
        let pb = indicatif::ProgressBar::new(total);
        pb.set_style(sty);
        for session in sessions.clone() {
            if first {
                (term_year, term, week) = crate::tools::term_year_detial(session);
                first = false;
            }
            let jie = crate::tools::now_to_jie(this);
            let live = Live::get_lives_by_time(session, term_year, term, week, week_day, jie);
            if let Ok(Some(live)) = live {
                lives_map.insert(session, live);
            }
            pb.inc(1)
        }
        pb.finish_with_message("获取直播号完成。");
        let mut lives = HashSet::new();
        for live in lives_map.values() {
            lives.insert(live.get_id());
        }
        let mut rooms = HashMap::new();
        let sty = indicatif::ProgressStyle::with_template(
            "获取地址中：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap();
        let pb = indicatif::ProgressBar::new(lives.len() as u64 * 2);
        pb.set_style(sty);
        pb.inc(0);
        for session in sessions.clone() {
            for live in lives {
                if let Some(room) = Room::get_rooms(session, &live).unwrap() {
                    pb.inc(1);
                    let video_path = room.get_live_video_path(session);
                    pb.inc(1);
                    rooms.insert(live, (room, video_path));
                } else {
                    pb.inc(2);
                }
            }
            break;
        }
        pb.finish_with_message("已获取直播地址。");
        let mut results = HashMap::new();
        for (session, live) in lives_map {
            if let Some((room, video_path)) = rooms.get(&live.get_id()) {
                results.insert(
                    session.get_uid(),
                    (session.get_stu_name(), room.clone(), video_path.clone()),
                );
            }
        }
        results
    }
}
