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

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::tools::{
    arc_into_inner_error_handler, json_parsing_error_handler, mutex_into_inner_error_handler,
    VideoPath,
};
use crate::{ProgressState, ProgressTracker, ProgressTrackerHolder};
use cxsign_user::Session;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Time_ {
    time: i64,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Lesson {
    #[serde(rename = "startTime")]
    start_time: Time_,
    id: i64,
}

impl Lesson {
    pub fn get_start_time(&self) -> i64 {
        self.start_time.time
    }
    pub fn get_live_id(&self) -> i64 {
        self.id
    }
    pub fn get_recording_url(
        session: &Session,
        live_id: i64,
    ) -> Result<VideoPath, Box<ureq::Error>> {
        crate::tools::get_recording_live_video_path(session, live_id)
    }
    pub fn get_all_lessons(session: &Session, live_id: i64) -> Result<Vec<i64>, Box<ureq::Error>> {
        let mut lessons: Vec<Lesson> = crate::protocol::list_single_course(session, live_id)?
            .into_json()
            .unwrap_or_else(json_parsing_error_handler);
        lessons.sort_by_key(|l| l.get_start_time());
        Ok(lessons.into_iter().map(|l| l.get_live_id()).collect())
    }
    pub fn get_recording_lives<P: ProgressTracker + 'static>(
        session: &Session,
        live_id: i64,
        multi: &impl ProgressTrackerHolder<P>,
    ) -> Result<HashMap<i64, VideoPath>, Box<ureq::Error>> {
        let lessons: Vec<Lesson> = crate::protocol::list_single_course(session, live_id)?
            .into_json()
            .unwrap_or_else(json_parsing_error_handler);
        let total = lessons.len();
        let thread_count = total / 64;
        let rest_count = total % 64;
        let pb = multi.init(total as u64, ProgressState::GetRecordingLives);
        let pb = Arc::new(Mutex::new(pb));
        let paths = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = Vec::new();
        for block in 0..64 {
            let ref_ = if block < rest_count {
                ((thread_count + 1) * block)..((thread_count + 1) * (block + 1))
            } else {
                (thread_count * block + rest_count)..(thread_count * (block + 1) + rest_count)
            };
            let session = (*session).clone();
            let paths = Arc::clone(&paths);
            let pb = Arc::clone(&pb);
            let mut lessons_ = vec![];
            for lesson in &lessons[ref_] {
                lessons_.push(lesson.clone())
            }
            let handle = std::thread::spawn(move || {
                for lesson in lessons_ {
                    if let Ok(path) = Lesson::get_recording_url(&session, lesson.get_live_id()) {
                        paths.lock().unwrap().insert(lesson.get_start_time(), path);
                    }
                    pb.lock().unwrap().inc(1);
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        let paths = Arc::into_inner(paths)
            .unwrap_or_else(arc_into_inner_error_handler)
            .into_inner()
            .unwrap_or_else(mutex_into_inner_error_handler);
        let pb = Arc::into_inner(pb)
            .unwrap_or_else(arc_into_inner_error_handler)
            .into_inner()
            .unwrap_or_else(mutex_into_inner_error_handler);
        pb.finish(ProgressState::GetRecordingLives);
        multi.remove_progress(&pb);
        Ok(paths)
    }
}
