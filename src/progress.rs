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

pub trait ProgressTrackerHolder<P: ProgressTracker> {
    fn init(&self, total: u64, data: ProgressState) -> P;
    fn remove_progress(&self, progress: &P);
}
pub trait ProgressTracker: Send + Sized {
    fn inc(&self, delta: u64);
    fn go_on(&self) -> bool {
        true
    }
    fn finish(&self, data: ProgressState);
}
#[non_exhaustive]
pub enum ProgressState {
    GetRecordingLives,
    GetLiveIds,
    GetLiveUrls,
    GetDeviceCodes,
}
