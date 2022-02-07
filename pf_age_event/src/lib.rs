use ndk::event::InputEvent;
#[derive(Debug)]
pub enum Event{
    SystemEvent(SystemEvent),
    InputEvent(InputEventWrapper),
}

#[derive(Debug)]
pub struct InputEventWrapper{
    ev :InputEvent,
}

unsafe impl Send for InputEventWrapper{}
unsafe impl Sync for InputEventWrapper{}


#[derive(Debug)]
pub enum SystemEvent{
    AndroidNativeWindowCreated,
    AndroidNativeWindowDestoryed,
    AndroidNativeInputQueueCreated,
    AndroidNativeInputQueueDestroyed,
}
