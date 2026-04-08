/*
 * Project: MultiSerial
 * Version: 0.1.0
 * Author: kong
 * Description: Serial port management and monitoring logic for MultiSerial.
 */

use serialport::{SerialPort, available_ports};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use anyhow::{Result, anyhow};
use std::io::Read;

// ── Charset ──────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Charset {
    Utf8,
    Gbk,
    Ascii,
    Latin1,
}

impl Charset {
    pub const ALL: &'static [Charset] = &[Charset::Utf8, Charset::Gbk, Charset::Ascii, Charset::Latin1];

    pub fn label(&self) -> &'static str {
        match self {
            Charset::Utf8   => "UTF-8",
            Charset::Gbk    => "GBK",
            Charset::Ascii  => "ASCII",
            Charset::Latin1 => "ISO-8859-1",
        }
    }

    pub fn decode(&self, bytes: &[u8]) -> String {
        let text = match self {
            Charset::Utf8  => String::from_utf8_lossy(bytes).to_string(),
            Charset::Ascii => bytes.iter().map(|&b| if b.is_ascii() { b as char } else { '?' }).collect(),
            Charset::Gbk   => {
                let (cow, _, _) = encoding_rs::GBK.decode(bytes);
                cow.into_owned()
            }
            Charset::Latin1 => {
                let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
                cow.into_owned()
            }
        };
        text
    }

    pub fn strip_ansi(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut iter = text.chars().peekable();
        while let Some(c) = iter.next() {
            // ANSI Escape Sequence: ESC [ ... m/K/etc
            if c == '\x1b' || c == '\u{001b}' {
                if let Some(&'[') = iter.peek() {
                    let _ = iter.next(); // skip '['
                    while let Some(&next) = iter.peek() {
                        let _ = iter.next();
                        if (next as u8) >= 0x40 && (next as u8) <= 0x7e {
                            break;
                        }
                    }
                    continue;
                }
            }
            
            // Strip non-printable control characters (0-31) except whitespace
            let code = c as u32;
            if code < 32 && c != '\n' && c != '\r' && c != '\t' {
                continue;
            }
            // Strip high-range "junk" characters often rendered as boxes
            if code == 0x7F || (code >= 0x80 && code <= 0x9F) {
                continue;
            }

            result.push(c);
        }
        result
    }
}

// ── Line Ending ──────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LineEnding {
    Lf,
    Cr,
    CrLf,
    None,
}

impl LineEnding {
    pub const ALL: &'static [LineEnding] = &[LineEnding::CrLf, LineEnding::Lf, LineEnding::Cr, LineEnding::None];

    pub fn label(&self) -> &'static str {
        match self {
            LineEnding::Lf   => "LF (\\n)",
            LineEnding::Cr   => "CR (\\r)",
            LineEnding::CrLf => "CR+LF (\\r\\n)",
            LineEnding::None => "None",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf   => "\n",
            LineEnding::Cr   => "\r",
            LineEnding::CrLf => "\r\n",
            LineEnding::None => "",
        }
    }
}

// ── PortConfig ───────────────────────────────────────────────────
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PortConfig {
    pub name: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: u8,     // 0=None 1=Odd 2=Even
    pub stop_bits: u8,  // 1 or 2
    pub flow_control: u8, // 0=None 1=Hardware 2=Software
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            baud_rate: 115200,
            data_bits: 8,
            parity: 0,
            stop_bits: 1,
            flow_control: 0,
        }
    }
}

impl PortConfig {
    pub fn sp_data_bits(&self) -> serialport::DataBits {
        match self.data_bits {
            5 => serialport::DataBits::Five,
            6 => serialport::DataBits::Six,
            7 => serialport::DataBits::Seven,
            _ => serialport::DataBits::Eight,
        }
    }
    pub fn sp_parity(&self) -> serialport::Parity {
        match self.parity {
            1 => serialport::Parity::Odd,
            2 => serialport::Parity::Even,
            _ => serialport::Parity::None,
        }
    }
    pub fn sp_stop_bits(&self) -> serialport::StopBits {
        match self.stop_bits {
            2 => serialport::StopBits::Two,
            _ => serialport::StopBits::One,
        }
    }
    pub fn sp_flow_control(&self) -> serialport::FlowControl {
        match self.flow_control {
            1 => serialport::FlowControl::Hardware,
            2 => serialport::FlowControl::Software,
            _ => serialport::FlowControl::None,
        }
    }

    pub fn parity_label(&self) -> &'static str {
        match self.parity { 1 => "Odd", 2 => "Even", _ => "None" }
    }
    pub fn stop_bits_label(&self) -> &'static str {
        match self.stop_bits { 2 => "2", _ => "1" }
    }
    pub fn flow_label(&self) -> &'static str {
        match self.flow_control { 1 => "Hardware", 2 => "Software", _ => "None" }
    }
}

// ── Log Entry ────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub text: String,
    pub raw: Vec<u8>,
}

// ── SerialManager ────────────────────────────────────────────────
pub struct SerialManager {
    pub logs: Arc<Mutex<Vec<LogEntry>>>,
    pub rx_count: Arc<Mutex<usize>>,
    pub tx_count: Arc<Mutex<usize>>,
    port: Option<Box<dyn SerialPort>>,
    is_running: Arc<Mutex<bool>>,
}

impl SerialManager {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
            rx_count: Arc::new(Mutex::new(0)),
            tx_count: Arc::new(Mutex::new(0)),
            port: None,
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn list_ports() -> Vec<String> {
        available_ports()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.port_name)
            .collect()
    }

    pub fn connect(&mut self, config: &PortConfig, charset: Charset) -> Result<()> {
        let port = serialport::new(&config.name, config.baud_rate)
            .data_bits(config.sp_data_bits())
            .parity(config.sp_parity())
            .stop_bits(config.sp_stop_bits())
            .flow_control(config.sp_flow_control())
            .timeout(Duration::from_millis(10))
            .open()?;

        let logs = Arc::clone(&self.logs);
        let rx_count = Arc::clone(&self.rx_count);
        let is_running = Arc::clone(&self.is_running);
        let cfg = config.clone();

        *is_running.lock().unwrap() = true;

        // The read thread owns port_clone and will auto-reconnect on error
        let mut port_clone = port.try_clone()?;

        thread::spawn(move || {
            let mut read_buf: Vec<u8> = vec![0; 4096];
            let mut line_buffer: Vec<u8> = Vec::new();
            loop {
                if !*is_running.lock().unwrap() {
                    break;
                }
                match port_clone.read(read_buf.as_mut_slice()) {
                    Ok(t) if t > 0 => {
                        let data = &read_buf[..t];
                        *rx_count.lock().unwrap() += t;
                        
                        for &b in data {
                            if b == b'\n' {
                                // Full line completed (\n)
                                let mut raw = std::mem::take(&mut line_buffer);
                                raw.push(b'\n'); // Keep the newline for raw representation
                                let mut text = charset.decode(&raw);
                                // Strip any trailing \r or \n for the display text
                                text = text.trim_end_matches(|c| c == '\r' || c == '\n').to_string();
                                
                                // Strip ANSI and check if it's junk
                                let stripped = Charset::strip_ansi(&text);
                                let trimmed = stripped.trim();
                                if trimmed.is_empty() && text.len() < 4 {
                                    continue;
                                }

                                let ts = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
                                let mut logs_lock = logs.lock().unwrap();
                                logs_lock.push(LogEntry { timestamp: ts, text, raw });
                                if logs_lock.len() > 5000 { logs_lock.drain(0..1000); }
                            } else if b == b'\r' {
                                // Handle \r as a terminator if not followed by \n (is handled next loop)
                                // But for simplicity, we just store it and wait for \n
                                // OR if we get a \r and then something else, we might want to split.
                                // Most serial devices use \r\n.
                                line_buffer.push(b);
                            } else {
                                line_buffer.push(b);
                                if line_buffer.len() > 8192 {
                                    let raw = std::mem::take(&mut line_buffer);
                                    let text = charset.decode(&raw);
                                    let ts = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
                                    let mut logs_lock = logs.lock().unwrap();
                                    logs_lock.push(LogEntry { timestamp: ts, text, raw });
                                    if logs_lock.len() > 5000 { logs_lock.drain(0..1000); }
                                }
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(_e) => {
                        // Device disconnected — log it and try to reconnect
                        {
                            let ts = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
                            let mut logs_lock = logs.lock().unwrap();
                            logs_lock.push(LogEntry {
                                timestamp: ts,
                                text: format!("[{}] ⚠ Device disconnected, auto-reconnecting…", cfg.name),
                                raw: Vec::new(),
                            });
                        }

                        // Retry loop
                        loop {
                            if !*is_running.lock().unwrap() {
                                return; // User closed the port
                            }
                            thread::sleep(Duration::from_millis(200));
                            match serialport::new(&cfg.name, cfg.baud_rate)
                                .data_bits(cfg.sp_data_bits())
                                .parity(cfg.sp_parity())
                                .stop_bits(cfg.sp_stop_bits())
                                .flow_control(cfg.sp_flow_control())
                                .timeout(Duration::from_millis(10))
                                .open()
                            {
                                Ok(new_port) => {
                                    match new_port.try_clone() {
                                        Ok(cloned) => {
                                            port_clone = cloned;
                                            let ts = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
                                            let mut logs_lock = logs.lock().unwrap();
                                            logs_lock.push(LogEntry {
                                                timestamp: ts,
                                                text: format!("[{}] ✓ Reconnected successfully", cfg.name),
                                                raw: Vec::new(),
                                            });
                                            break; // Back to read loop
                                        }
                                        Err(_) => continue,
                                    }
                                }
                                Err(_) => continue, // Still disconnected — wait & retry
                            }
                        }
                    }
                }
            }
            *is_running.lock().unwrap() = false;
        });

        self.port = Some(port);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        *self.is_running.lock().unwrap() = false;
        self.port = None;
    }

    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        if let Some(port) = &mut self.port {
            port.write_all(data)?;
            *self.tx_count.lock().unwrap() += data.len();
            Ok(())
        } else {
            Err(anyhow!("Port not connected"))
        }
    }
}
