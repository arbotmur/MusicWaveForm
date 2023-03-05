use std::env;
use std::path::Path;
use hound::{WavReader, Error};
use image::{ImageBuffer, Rgb};
use rustfft::num_complex::Complex;
use rustfft::{FftDirection, FftPlanner};
use std::error::Error as StdError;
use rustfft::num_traits::Zero;

fn generate_waveform_image(audio_file_path: &str, image_file_path: &str) -> Result<(), Box<dyn StdError>> {
    // Ouvre le fichier audio et récupère les échantillons
    let mut reader = WavReader::open(audio_file_path)?;
    let samples: Vec<i32> = reader.samples().collect::<Result<Vec<i32>, Error>>()?;

    // Calcule la FFT des échantillons
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft(samples.len(), FftDirection::Forward);
    let mut spectrum = vec![Complex::zero(); samples.len()];
    for (i, sample) in samples.iter().enumerate() {
        spectrum[i] = Complex::new(*sample as f32, 0.0);
    }
    fft.process(&mut spectrum);

    // Crée une image à partir de la waveform
    let width = 800;
    let height = 200;
    let mut waveform = ImageBuffer::new(width, height);
    let max_amplitude = 2.0_f32.powf(reader.spec().bits_per_sample as f32 - 1.0);
    let samples_per_pixel = spectrum.len() as u32 / width;
    for x in 0..width {
        let mut sum = 0.0;
        for i in 0..samples_per_pixel {
            sum += spectrum[(x * samples_per_pixel + i) as usize].norm();
        }
        let avg_amplitude = sum / samples_per_pixel as f32;
        let pixel_height = (avg_amplitude / max_amplitude * height as f32) as u64;
        let color = get_color(pixel_height, height as u64);
        for y in height - (pixel_height as u32)..height {
            waveform.put_pixel(x, y, Rgb(color));
        }
    }

    // Enregistre l'image dans un fichier JPEG
    waveform.save(image_file_path)?;

    Ok(())
}

// Retourne un tableau de trois u8 représentant une couleur en fonction de la hauteur du pixel.
fn get_color(pixel_height: u64, height: u64) -> [u8; 3] {
    let percent = pixel_height as f64 / height as f64;
    let r = (255.0 * percent) as u8;
    let g = (255.0 * (1.0 - percent)) as u8;
    let b = (255.0 * percent.powf(2.0)) as u8;
    [r, g, b]
}

fn main() {
    // Récupère le premier argument en ligne de commande comme chemin vers le fichier audio Wav
    let args: Vec<String> = env::args().collect();
    let audio_file_path = match args.get(1) {
        Some(path) => path,
        None => {
            eprintln!("Veuillez spécifier un chemin vers un fichier audio Wav en argument.");
            return;
        }
    };

    // Vérifie que le fichier existe et est bien un fichier Wav
    let audio_file_path = Path::new(audio_file_path);
    if !audio_file_path.exists() {
        eprintln!("Le fichier spécifié n'existe pas.");
        return;
    }
    
    if audio_file_path.extension().unwrap_or_default() != "wav" {
        eprintln!("Le fichier spécifié n'est pas un fichier Wav.");
        return;
    }

    // Génère la waveform et affiche un message de succès
    let image_file_path = "waveform.jpg";
    match generate_waveform_image(audio_file_path.to_str().unwrap(), image_file_path) {
        Ok(_) => println!("Waveform générée avec succès : {}", image_file_path),
        Err(error) => eprintln!("Erreur lors de la génération de la waveform : {}", error),
    };
}