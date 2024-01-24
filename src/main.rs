use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use rand::thread_rng;
use rand::distributions::{Distribution, Uniform};
use warp::Filter;
use std::path::PathBuf;

#[tokio::main]
async fn main() {

    //Create a channel to communicate between threads
    let (tx, rx) = std::sync::mpsc::channel::<i32>();
    let rx_shared = Arc::new(Mutex::new(rx));

    //spawn a thread for generating the random numbers
    {
        let tx = tx.clone();
        thread::spawn(move || {
            generate(tx);
        });
    }

    // route for serving the html file
    let html_route = warp::path::end().map(|| {
        let html_path = PathBuf::from("static/index.html"); 
        warp::reply::html(std::fs::read_to_string(html_path).unwrap())
    });

    //ceate a warp route
    let get_move_route = warp::path!("get_move")
        .map({
            let rx_shared = Arc::clone(&rx_shared); 
            move || {
                let move_value = {
                    let rx = rx_shared.lock().unwrap();
                    rx.recv().unwrap_or_else(|_| 0)
                };
            
                warp::reply::with_header(
                    move_value.to_string(),
                    "Content-Type",
                    "text/plain"
                )
            }
        });
    
    let routes = html_route.or(get_move_route);

    warp::serve(routes)
        .run(([127,0,0,1], 3030))
        .await;

}

fn generate(tx: mpsc::Sender<i32>) {


    loop {
        let move_value = get_move();

        //send to the channel
        tx.send(move_value).unwrap();

    }
}

fn get_move() -> i32 {
    let mut rng = thread_rng();
    let moves = Uniform::new_inclusive(1, 12);
    moves.sample(&mut rng)

}
