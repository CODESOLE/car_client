use mio::{net::TcpStream, Events, Interest, Poll, Token};
use std::{
    fmt::{self, Display},
    io::{self, Read, Write},
    net::Shutdown,
    str::from_utf8,
    str::FromStr,
    thread,
    time::Duration,
};

#[derive(Debug, PartialEq)]
pub struct Car {
    pub pos: (i32, i32),
    pub target: (i32, i32),
}

impl Car {
    pub fn new(pos: (i32, i32), target: (i32, i32)) -> Self {
        Self { pos, target }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseCarErr;

impl FromStr for Car {
    type Err = ParseCarErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.split_once(':'))
            .ok_or(ParseCarErr)?;

        let (x1, y1) = x
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.split_once(','))
            .ok_or(ParseCarErr)?;

        let (x2, y2) = y
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.split_once(','))
            .ok_or(ParseCarErr)?;

        let x1_fromstr = x1.parse::<i32>().map_err(|_| ParseCarErr)?;
        let y1_fromstr = y1.parse::<i32>().map_err(|_| ParseCarErr)?;
        let x2_fromstr = x2.parse::<i32>().map_err(|_| ParseCarErr)?;
        let y2_fromstr = y2.parse::<i32>().map_err(|_| ParseCarErr)?;

        Ok(Car {
            pos: (x1_fromstr, y1_fromstr),
            target: (x2_fromstr, y2_fromstr),
        })
    }
}

impl Display for Car {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(({},{}):({},{}))",
            self.pos.0, self.pos.1, self.target.0, self.target.1
        )
    }
}

fn main() -> io::Result<()> {
    const MAP: &str = "#..#######
#..#..#..#
#..#..#..#
#..#.....#
#..#.....#
#..####..#
#........#
##########";
    let mut map_result = String::new();

    let mut iteration_count: usize = 0;
    const CLIENT: Token = Token(0);
    let mut client = TcpStream::connect("127.0.0.1:9123".parse().unwrap())?;
    let mut car = Car::new((1, 1), (4, 1));
    let mut poll = Poll::new()?;
    poll.registry()
        .register(&mut client, CLIENT, Interest::WRITABLE)?;

    for (i, x) in MAP.lines().enumerate() {
        for (j, y) in x.chars().enumerate() {
            if car.pos.0 as usize == i && car.pos.1 as usize == j {
                map_result.push('S');
            } else if car.target.0 as usize == j && car.target.1 as usize == i {
                map_result.push('E');
            } else {
                map_result.push(y);
            }
        }
        map_result.push('\n');
    }

    let mut events = Events::with_capacity(1);
    'exit: loop {
        thread::sleep(Duration::from_millis(500));
        poll.poll(&mut events, None)?;
        for event in events.iter() {
            if car.pos == car.target {
                poll.registry().deregister(&mut client)?;
                TcpStream::shutdown(&client, Shutdown::Both)?;
                break 'exit;
            }
            match event.token() {
                CLIENT => {
                    if event.is_writable() {
                        client.write(car.to_string().as_bytes())?;
                        poll.registry()
                            .reregister(&mut client, CLIENT, Interest::READABLE)?;
                    }

                    if event.is_readable() {
                        let mut data = vec![0; 13];
                        client.read(&mut data)?;
                        let temp_car = from_utf8(&data).unwrap().parse::<Car>().unwrap();

                        for (i, x) in MAP.lines().enumerate() {
                            for (j, _) in x.chars().enumerate() {
                                if temp_car.pos.0 as usize == j && temp_car.pos.1 as usize == i {
                                    if temp_car.pos != temp_car.target {
                                        map_result
                                            .replace_range((11 * i + j)..(11 * i + j + 1), "@");
                                    }
                                }
                            }
                        }

                        car.pos.0 = temp_car.pos.0;
                        car.pos.1 = temp_car.pos.1;
                        iteration_count += 1;
                        println!("[Iteration {}]   {}", iteration_count, car);
                        poll.registry()
                            .reregister(&mut client, CLIENT, Interest::WRITABLE)?;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    println!("{}", map_result);
    Ok(())
}
