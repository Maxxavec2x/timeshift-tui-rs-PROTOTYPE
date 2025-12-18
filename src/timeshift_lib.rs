use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fmt;
use std::process::Command;
use std::str;

#[derive(Debug)]
pub enum TimeshiftError {
    DeleteError,
}

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq)]
pub struct Device {
    num: u8,
    device_name: String,
    size: String,
    device_type: String, //should be an enum, will do it later
    label: String, // I legit don't know what that is, mine is always left blank on my system, and I
                   // do not find the documentation (tell me if you know what that is)
}

impl Device {
    pub fn new(
        num: u8,
        device_name: String,
        size: String, // I use the size in String because its a float, and float cannot use the Eq
        // trait (maybe there is a workaround but I don't really need the size in f32
        // anyway)
        device_type: String,
        label: String,
    ) -> Self {
        Device {
            num,
            device_name,
            size,
            device_type,
            label,
        }
    }
}
impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | {} | {} | {} | {}",
            self.num, self.device_name, self.size, self.device_type, self.label
        )
    }
}

#[derive(Debug, Default, Clone)] // We cannot use the copy trait because there is a String in our
// struct (:/)
pub struct Snapshot {
    num: u8,

    pub name: String,

    tags: char,
    description: String,
}

impl Snapshot {
    pub fn new(num: u8, name: String, tags: char, description: String) -> Self {
        Snapshot {
            num,
            name,
            tags,
            description,
        }
    }
}
impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | {} | {} | {}",
            self.num, self.name, self.tags, self.description
        )
    }
}

#[derive(Debug, Default)]
pub struct Timeshift {
    //why did i create such a monster
    pub devices_map: HashMap<Device, Vec<Snapshot>>,
    pub devices_map_by_name: HashMap<String, Vec<Snapshot>>,
}

pub enum DeviceOrSnapshot {
    Device(Device),
    Snapshot(Snapshot),
}

impl Timeshift {
    pub fn new() -> Self {
        let mut devices_map: HashMap<Device, Vec<Snapshot>> = HashMap::new();
        let mut devices_map_by_name: HashMap<String, Vec<Snapshot>> = HashMap::new();
        let devices: Vec<Device> = Self::get_devices();
        for device in devices {
            devices_map.insert(device.clone(), Self::get_snapshots(device.clone()));
            devices_map_by_name.insert(device.clone().device_name, Self::get_snapshots(device));
            // Le fait de devoir faire des device.clone parce que je suis un noob avec les lifetime
            // est cursed, mais je réglerais ça + tard
            // HAHHAHAHAH C'EST LA GUERRE DES CLONES (vous l'avez ?)
        }

        Timeshift {
            devices_map,
            devices_map_by_name,
        }
    }

    pub fn get_snapshots(device: Device) -> Vec<Snapshot> {
        let output = Command::new("sudo")
            .arg("timeshift")
            .arg("--list")
            .arg("--snapshot-device")
            .arg(device.device_name)
            .output()
            .expect("Couldn't get snapshots list");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let result: Vec<Snapshot> = Self::parse_output(stdout.to_string(), "Snapshot")
            .into_iter()
            .map(|item| match item {
                DeviceOrSnapshot::Snapshot(snapshot) => snapshot,
                DeviceOrSnapshot::Device(_) => panic!("Expected Snapshot, got Device"),
            })
            .collect();
        result
    }

    pub fn get_devices() -> Vec<Device> {
        let output = Command::new("sudo")
            .arg("timeshift")
            .arg("--list-devices")
            .output()
            .expect("Couldn't get device list");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let result: Vec<Device> = Self::parse_output(stdout.to_string(), "Device")
            .into_iter()
            .map(|item| match item {
                DeviceOrSnapshot::Device(device) => device,
                DeviceOrSnapshot::Snapshot(_) => panic!("Expected Device, got snapshot"),
            })
            .collect();

        if result.is_empty() {
            panic!("No devices found");
        }
        println!("RESULT: {:?}", &result);
        result
    }

    // Le but de cette fonction est de généraliser le parsing des output de timeshift
    // I found out that the timeshift command always return dashes, so I exploit that.
    // I don't know if generalizing the output with an enum is good practice, but why not try ? We
    // have only one life after all :)
    pub fn parse_output(s: String, t: &str) -> Vec<DeviceOrSnapshot> {
        let mut result: Vec<DeviceOrSnapshot> = Vec::new();
        let lines_after_separator = s
            .split_once("----")
            .map(|(_, after)| after.lines().skip(1)) // skip(1) pour ignorer la fin de la ligne des dashes
            .into_iter()
            .flatten()
            .filter(|line| !line.trim().is_empty()); // Filtre les lignes vides !
        for line in lines_after_separator {
            if t == "Device" {
                let parts: Vec<&str> = line.split_whitespace().collect();
                println!("Processing device with parts: {:?}", parts);
                let num = parts[0].parse::<u8>().expect("Could not parse num");
                let device_name = parts[2].to_string();
                let size = parts[3].to_string();
                let device_type = parts[4].to_string();
                let label = "".to_string();
                // On récupère tout le
                // reste (heuresement que
                // le truc est à la fin)

                result.push(DeviceOrSnapshot::Device(Device::new(
                    num,
                    device_name,
                    size,
                    device_type,
                    label,
                )));
            } else if t == "Snapshot" {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 5 {
                    // panic!("Invalid snapshot format");
                }
                let num = parts[0].parse::<u8>().expect("Could not parse num");
                let name = String::from(parts[2]);
                let tags = parts[3].parse::<char>().expect("could not parse Tags");
                let description = parts[4..].join(" ").to_string();
                // On récupère tout le
                // reste (heuresement que
                // le truc est à la fin)
                result.push(DeviceOrSnapshot::Snapshot(Snapshot::new(
                    num,
                    name,
                    tags,
                    description,
                )));
            }
        }
        result
    }

    pub fn delete_snapshot(snapshot_name: &str) -> Result<()> {
        let output = Command::new("sudo")
            .arg("timeshift")
            .arg("--delete")
            .arg("--snapshot")
            .arg(snapshot_name)
            .output()
            .context("Failed to execute timeshift command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "Timeshift delete failed with exit code {:?}: {}",
                output.status.code(),
                stderr
            );
        }

        Ok(())
    }
}
