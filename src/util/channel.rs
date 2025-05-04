#[derive(Debug)]
pub struct Broadcast<T> {
    channels: Vec<crossbeam_channel::Sender<T>>,
}

impl<T: std::marker::Send + Clone + 'static> Broadcast<T> {
    pub fn new() -> Broadcast<T> {
        let b = Broadcast {
            channels: Vec::new(),
        };

        b
    }

    pub fn send(&self, data: T) {
        for channel in &self.channels {
            channel.send(data.clone()).unwrap();
        }
    }

    pub fn subscribe(&mut self) -> crossbeam_channel::Receiver<T> {
        let (tx, rx) = crossbeam_channel::unbounded::<T>();
        self.channels.push(tx);
        rx
    }
}
