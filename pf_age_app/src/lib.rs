use std::time;
use std::thread;
pub struct App {

}

impl App {
    pub fn new()->Self{
        Self{}
    }
    pub fn run() {
        loop{
            thread::sleep(time)
        }
    }
}
