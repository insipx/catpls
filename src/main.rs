mod bot;
mod rest;
mod content;

fn main() {
    std::thread::scope(|s| {
        s.spawn(|| {
            rest::web_server()
        });
    });
}
