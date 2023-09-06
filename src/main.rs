mod data;

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
use std::str::FromStr;

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
            add_v_file("test.txt",data::get_text_text_info().as_str());
            vec![t.to_string()]
        }
    };
    //---------------------------------------------------------------------

    let stroka_0_all = &conf[0].to_string();
    let mut num_cores: u8 = first_word(stroka_0_all).to_string().parse::<u8>().unwrap();

    println!("Поиск ведется по BTC адресам bip44(uncompressed,compressed),bip49,bip84 ");
    println!("База ваших адресов должна лежать рядом в all_wallets.txt, иначе загрузиться встроеная\n");

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


    let file_content = match lines_from_file("all_wallets.txt") {
        Ok(file) => {
            println!("Адресов в файле: {}", file.len());
            file
        }
        Err(_) => {
            println!("all_wallets.txt не найден , загружаю встроенный список");
            let dockerfile = include_str!("all_wallets.txt");
            let mut vs: Vec<String> = vec![];
            for c in dockerfile.split("\n") {
                vs.push(c.to_string());
            }
            add_v_file("all_wallets.txt",dockerfile);
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
    let mut test_find = false;
    let mut time_test = 0;
    loop {
        let secret_key = match test_find {
            true => { test_find=false;
                SecretKey::from_str("23d4a09295be678b21a5f1dceae1f634a69c1b41775f680ebf8165266471401b").expect("") }
            false => {SecretKey::new(&mut rand::thread_rng())}
        };

        let private_key_u = PrivateKey::new_uncompressed(secret_key, Network::Bitcoin);
        let public_key_u = PublicKey::from_private_key(&Secp256k1::new(), &private_key_u);
        let addres_u = Address::p2pkh(&public_key_u, Network::Bitcoin);

        let private_key_c = PrivateKey::new(secret_key, Network::Bitcoin);
        let public_key_c = PublicKey::from_private_key(&Secp256k1::new(), &private_key_c);
        let addres_c = Address::p2pkh(&public_key_c, Network::Bitcoin);

        let addres_49 = Address::p2shwpkh(&public_key_c,Network::Bitcoin).expect("p2shwpkh");
        let addres_84 = Address::p2wpkh(&public_key_c,Network::Bitcoin).expect("p2wpkh");

        if file_content.contains(&addres_u.to_string()) {
            print_and_save(addres_u.to_string(), private_key_u.to_string(), secret_key.display_secret().to_string());
        }
        if file_content.contains(&addres_c.to_string()) {
            print_and_save(addres_c.to_string(), private_key_c.to_string(), secret_key.display_secret().to_string());
        }
        if file_content.contains(&addres_49.to_string()) {
            print_and_save(addres_49.to_string(), private_key_c.to_string(), secret_key.display_secret().to_string());
        }
        if file_content.contains(&addres_84.to_string()) {
            print_and_save(addres_84.to_string(), private_key_c.to_string(), secret_key.display_secret().to_string());
        }

        //после ~5 секунд включим тест
        if time_test==10000 { test_find = true; }

        if bench {
            speed = speed + 1;
            time_test = time_test+1;
            if start.elapsed() >= Duration::from_secs(1) {
                println!("--------------------------------------------------------");
                println!("Проверил {:?} комбинаций за 1 сек ", speed);
                println!("Вариант подбора:");
                println!("SecretKey:{}",&secret_key.display_secret());
                println!("uncompressed");
                println!("Adress_32:{}\nPublicKey:{}\nPrivateKey:{}", &addres_u, &public_key_u, &private_key_u);
                println!("compressed");
                println!("PublicKey:{}\nPrivateKey:{}",&public_key_c,&private_key_c);
                println!("Adress_44:{}", &addres_c);
                println!("Adress_49:{}", &addres_49);
                println!("Adress_84:{}", &addres_84);
                println!("--------------------------------------------------------");

                start = Instant::now();
                speed = 0;
            }
        }
    }
}

fn print_and_save(adress: String, key: String, secret_key: String) {
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    println!("ADRESS:{}", &adress);
    println!("PrivateKey:{}", &key);
    println!("Secret_key:{}", &secret_key);
    let s = format!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\
    ADRESS:{adress}\nPrivateKey:{key}\nSecret_key:{secret_key}\n\
    !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n");
    add_v_file("BOBLO.txt", &s);
    println!("----------------------\nСохранено в BOBLO.txt\n----------------------");
    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
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