extern crate bitcoin;
extern crate secp256k1;
extern crate num_cpus;

use std::{
    collections::HashSet,
    io,
    fs::{OpenOptions},
    fs::File,
    io::Write,
    time::Instant,
    time::Duration,
    io::{BufRead, BufReader},
    path::Path,
};

use secp256k1::{rand, Secp256k1, SecretKey};
use bitcoin::{network::constants::Network, PrivateKey, Address, PublicKey};
use std::sync::{Arc, RwLock, RwLockReadGuard};

use tokio::task;

//большенство кода скопипастил от сюда
//https://github.com/a137x/plutus-rustus/blob/master/src/main.rs
#[tokio::main]
async fn main() {
    let file_cong = "conf_find_legasy.txt";
    //Чтение настроек, и если их нет создадим
    //-----------------------------------------------------------------
    let conf = match lines_from_file(&file_cong) {
        Ok(text) => {
            println!("Параметры загружены\n");
            text
        }
        Err(_) => {
            println!("Параметры не найдены , создание и установка в режим измерения скорости\n");
            let t = format!("0 -Количество ядер от 0 до {} (0 - режим измерения скорости)", num_cpus::get());
            add_v_file(&file_cong, &t);
            vec![t.to_string()]
        }
    };
    //---------------------------------------------------------------------

    let stroka_0_all = &conf[0].to_string();
    let mut num_cores: u8 = first_word(stroka_0_all).to_string().parse::<u8>().unwrap();

    println!("Поиск ведется по адресу(1...) старому и новому");
    println!("База ваших адресов должна лежать рядом в bip44_wallets.txt, иначе загрузиться встроеная\n");

    println!("Если чудо произойдёт выведется результат в консоль и ключ запишеться BOBLO.txt\n\
    (будет лежать рядом, создавать не обязательно)\n");

    let mut bench = false;
    if num_cores == 0 {
        println!("---------------------------------------------------");
        println!("--------------Режим измерения скорости-------------");
        println!("--------------------------------------------------");
        bench = true;
        num_cores = 1;
    }
    println!("Из {} логических процессоров задействовано:{} \n", num_cpus::get(), num_cores);


    let file_content = match lines_from_file("bip44_wallets.txt") {
        Ok(file) => {
            println!("Адресов в файле: {}", file.len());
            file
        }
        Err(_) => {
            println!("bip44_wallets.txt не найден , загружаю встроенный список");
            let dockerfile = include_str!("bip44_wallets.txt");
            let mut vs: Vec<String> = vec![];
            for c in dockerfile.split("\n") {
                vs.push(c.to_string());
            }
            vs
        }
    };

    let mut database = HashSet::new();
    for addres in file_content.iter() {
        database.insert(addres.to_string());
    }

    println!("Загруженно в базу {:?} адресов.\n", database.len());
    let database_ = Arc::new(RwLock::new(database));


    for _ in 0..num_cores {
        let clone_database_ = Arc::clone(&database_);
        task::spawn_blocking(move || {
            let current_core = std::thread::current().id();
            let db = clone_database_.read().unwrap();
            println!("Процесс {:?} запущен\n", current_core);
            process(&db, bench);
        });
    }
}

fn process(file_content: &RwLockReadGuard<HashSet<String>>, bench: bool) {
    let mut start = Instant::now();
    let mut speed: u32 = 0;

    loop {
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        let private_key_uncompressed = PrivateKey::new_uncompressed(secret_key, Network::Bitcoin);
        let public_key_uncompressed = PublicKey::from_private_key(&Secp256k1::new(), &private_key_uncompressed);
        let address_uncompressed = Address::p2pkh(&public_key_uncompressed, Network::Bitcoin);

        let private_key_compressed = PrivateKey::new(secret_key, Network::Bitcoin);
        let public_key_compressed = PublicKey::from_private_key(&Secp256k1::new(), &private_key_compressed);
        let address_compressed = Address::p2pkh(&public_key_compressed, Network::Bitcoin);


        if file_content.contains(&address_uncompressed.to_string()) {
            print_and_save(address_uncompressed.to_string(), private_key_uncompressed.to_string(), secret_key.display_secret().to_string());
        }
        if file_content.contains(&address_compressed.to_string()) {
            print_and_save(address_compressed.to_string(), private_key_compressed.to_string(), secret_key.display_secret().to_string());
        }

        if bench {
            speed = speed + 1;
            if start.elapsed() >= Duration::from_secs(1) {
                let address = Address::p2pkh(&public_key_uncompressed, Network::Bitcoin);
                println!("--------------------------------------------------------");
                println!("Проверил {:?} комбинаций за 1 сек ", speed);
                println!("Вариант подбора:");
                println!("Adress:{}\nPublicKey:{}\nPrivateKey:{}", address, &public_key_uncompressed, &private_key_uncompressed);
                println!("ADRESS:{}\nPublicKeyc:{}\nPrivateKey:{}", address_compressed,private_key_compressed,public_key_compressed);
                println!("--------------------------------------------------------");
                start = Instant::now();
                speed = 0;
            }
        }
    }
}

fn print_and_save(adress: String, key: String, secret_key: String) {
    println!("ADRESS:{}", &adress);
    println!("PrivateKey:{}", &key);
    println!("Secret_key:{}", &secret_key);
    let s = format!("ADRESS:{} PrivateKey:{}\nSecret_key{}\n", &adress, &key, &secret_key);
    add_v_file("BOBLO.txt", &s);
    println!("-----------\n-----------\nСохранено в BOBLO.txt-----------\n-----------\n");
}

fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filename)?).lines().collect()
}

fn add_v_file(name: &str, data: &str) {
    OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(name)
        .expect("cannot open file")
        .write(data.as_bytes())
        .expect("write failed");
}

fn first_word(s: &String) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}