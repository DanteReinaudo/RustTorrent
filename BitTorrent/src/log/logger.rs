use crate::log::error::LogError;
use chrono::offset::Local;
use std::fs::create_dir;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::Receiver;

/******************************************************************************************/
/*                                      LOGGER                                            */
/******************************************************************************************/

/// Estructura encargada de loguear las operaciones
#[derive(Debug)]
#[allow(dead_code)]
pub struct Logger {
    pub file: File,
    pub path: String,
    pub rx: Receiver<String>,
}

#[allow(dead_code)]
impl Logger {
    /// El logger se inicializa con un directorio de logs, el nombre del archivo a loguear, y el receiver de un channel para escuchar los mensajes.
    /// Si no existe el directorio lo crea.
    pub fn new(log_path: &str, file_name: &str, rx: Receiver<String>) -> Result<Logger, LogError> {
        let folder = Path::new(log_path);
        if !folder.is_dir() {
            let _f = create_dir(folder).or(Err(LogError::FileCreationError))?;
        }

        let path_name = log_path.to_string() + "/" + file_name + ".log";
        let path = Path::new(&path_name);
        if !path.exists() {
            let _f = File::create(path).or(Err(LogError::FileCreationError))?;
        }
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .or(Err(LogError::FileCreationError))?;
        Ok(Logger {
            file,
            path: path_name,
            rx,
        })
    }

    /// Escribe el mensaje en el archivo de log agregando el horario
    fn write_log(&mut self, message: &str) -> Result<bool, LogError> {
        let time = Local::now().to_string();
        let log_message = Self::make_message(&time, message);
        writeln!(self.file, "{}", log_message).or(Err(LogError::WriteFileError))?;
        Ok(true)
    }

    /// Recibe el mensaje y el tiempo y los une.
    fn make_message(time: &str, message: &str) -> String {
        let mut log_message = String::from("");
        log_message.push_str(&*("[".to_owned() + time + "] " + message));
        log_message
    }

    /// Itera el receiver para loguear las operaciones.
    pub fn listening(&mut self) -> Result<(), LogError> {
        for received in &self.rx {
            let time = Local::now().to_string();
            let log_message = Self::make_message(&time, &received);
            writeln!(self.file, "{}", log_message).or(Err(LogError::WriteFileError))?;
        }
        Ok(())
    }

    /// Limpia el archivo de logs
    fn clear(&self) -> Result<bool, LogError> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
            .or(Err(LogError::OpenFileError))?;
        Ok(true)
    }
}

#[cfg(test)]
mod logger_should {
    use super::*;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn initialize_logger() {
        let (_tx, rx) = mpsc::channel();
        let path = String::from("./logs/prueba.log");
        let logger = Logger::new("./logs", "prueba", rx).unwrap();
        assert_eq!(logger.path, path);
    }

    #[test]
    #[ignore]
    fn write_file() {
        let (_tx, rx) = mpsc::channel();
        let path = String::from("./logs/prueba.log");
        let mut logger = Logger::new("./logs", "prueba", rx).unwrap();
        let message = "cayolanochee";
        logger.write_log(message).unwrap();

        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut lineas: Vec<String> = Vec::new();
        for line in reader.lines() {
            let line = line.unwrap();
            lineas.push(line.to_string());
        }
        assert!(lineas[0].contains(&message));
    }

    #[test]
    #[ignore]
    fn clear_file() {
        let (_tx, rx) = mpsc::channel();
        let path = String::from("./logs/prueba.log");
        let logger = Logger::new("./logs", "prueba", rx).unwrap();
        logger.clear().expect("Error");
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut lineas: Vec<String> = Vec::new();
        for line in reader.lines() {
            let line = line.unwrap();
            lineas.push(line.to_string());
        }
        let expected: Vec<String> = Vec::new();
        assert_eq!(lineas, expected);
    }

    #[test]
    fn make_message() {
        let time = Local::now().to_string();
        let mut expected = String::from("");
        expected.push_str(&*("[".to_owned() + &time + "] " + "Pincho el main"));
        let message = Logger::make_message(&time, "Pincho el main");
        assert_eq!(expected, message)
    }

    #[test]
    #[ignore]
    fn receive_from_thread() {
        let (tx, rx) = mpsc::channel();
        let mut logger = Logger::new("./logs", "prueba", rx).unwrap();
        let vals = vec![
            String::from("Mensaje 1"),
            String::from("Mensaje 2"),
            String::from("Mensaje 3"),
            String::from("Mensaje 4"),
        ];
        let vals_clone = vals.clone();
        let sender = thread::spawn(move || {
            for val in vals_clone {
                tx.send(val).unwrap();
                thread::sleep(Duration::from_secs(1));
            }
        });

        let _result = logger.listening().unwrap();
        sender.join().unwrap();

        let path = String::from("./logs/prueba.log");
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        for (i, line) in reader.lines().into_iter().enumerate() {
            let line = line.unwrap();
            assert!(line.to_string().contains(&vals[i]));
        }
    }
}
