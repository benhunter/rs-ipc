use ipc_channel::ipc::{self, *};
use std::env;
use std::process::{self, Command};

pub fn spawn_server(_ignored: &str, arg: &str) -> process::Child {
    Command::new(env::current_exe().unwrap())
        .arg(arg)
        .spawn()
        .expect("failed to execute server process")
}

type Data = (String, String);
type Bootstrap = (IpcSender<Data>, IpcReceiver<Data>);

fn main() {
    if env::args().len() == 1 {
        println!("Inside process_handler");
        let (server0, server_name0) = IpcOneShotServer::<Bootstrap>::new().unwrap();
        println!("server_name0 in process_handler: {:?}", server_name0);
        let guiserver = spawn_server(
            "path/to/script.sh",
            &server_name0
        );
        let (_receiver, (sender, receiver)): (IpcReceiver<Bootstrap>, Bootstrap) = server0.accept().unwrap();
        let data = ("KeyFromParent".to_string(), "36".to_string());
        sender.send(data.clone()).unwrap();
        println!("parent sent {:?}", data);
        let received = receiver.recv().unwrap();
        println!("parent received {:?}", received);

        loop {
            let received = receiver.recv().unwrap();
            println!("parent received len={:?}", received.1.len());
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

    } else {

        println!("Inside spawned process");
        let args: Vec<String> = env::args().collect();
        let (to_child, from_parent): (IpcSender<Data>, IpcReceiver<Data>) = ipc::channel().unwrap();
        let (to_parent, from_child): (IpcSender<Data>, IpcReceiver<Data>) = ipc::channel().unwrap();
        let bootstrap = IpcSender::connect(args[1].clone()).unwrap();
        bootstrap.send((to_child, from_child)).unwrap();
        let data = from_parent.recv().unwrap();
        println!("child received {:?}", data);
        to_parent.send(("KeyFromChild".to_string(), "8".to_string()));

        let mut count = 1;
        loop {
            let val = std::iter::repeat("X").take(count).collect::<String>();
            let data = ("send count=".to_string(), val);
            println!("child sent. count={:?}", count);
            to_parent.send(data);
            std::thread::sleep(std::time::Duration::from_millis(1));
            count *= 2;
        }

    }
}
