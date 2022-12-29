#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use core::fmt;
use std::thread;
use std::thread::available_parallelism;
use crossbeam_channel::unbounded;

use eframe::egui;

mod calculate;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::*;
use egui::plot::Line;
use egui::plot::Plot;



// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "integral calculator",
        native_options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "integral_calculator", // hardcode it
            web_options,
            Box::new(|_cc| Box::new(MyApp::default())),
        )
        .await
        .expect("failed to start eframe");
    });
}

struct MyApp {
    equ: String,
    start: String,
    end: String,
    result: String,
    riemann_sum_type: RiemannSum,
    rectangle_count: i32,
    rectangle_heights: Vec<[f64;2]>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            equ: "sin(x)".to_string(),
            start: "0.0".to_string(),
            end: "360.0".to_string(),
            result: "20.0".to_string(),
            riemann_sum_type: RiemannSum::Left,
            rectangle_count: 10000,
            rectangle_heights: vec![[0.0,0.0]],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RiemannSum {
    Left,
    Right,
    Midpoint,
}
impl std::fmt::Display for RiemannSum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RiemannSum::Left => write!(f, "Left"),
            RiemannSum::Right => write!(f, "Right"),
            RiemannSum::Midpoint => write!(f, "Midpoint"),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Get current context style
            let mut style = (*ctx.style()).clone();

            // Redefine text_styles
            style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Name("Heading2".into()), FontId::new(25.0, Proportional)),
            (Name("Context".into()), FontId::new(23.0, Proportional)),
            (Body, FontId::new(28.0, Proportional)),
            (Monospace, FontId::new(14.0, Proportional)),
            (Button, FontId::new(26.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
            ].into();

            // Mutate global style with above changes
            ctx.set_style(style);

            ui.heading("Integral Calculator");
            
            ui.horizontal(|ui| {
                ui.vertical(|ui|{
                    ui.horizontal(|ui|{
                        ui.label("End:");
                        ui.add_sized([60.,20.], egui::TextEdit::singleline(&mut self.end));
                    });
                    ui.horizontal(|ui|{
                        ui.label("Start:");
                        ui.add_sized([60.,20.], egui::TextEdit::singleline(&mut self.start));
                        
                    });
                });
                ui.text_edit_singleline(&mut self.equ);
            });

            ui.label("Riemann Sum type");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.riemann_sum_type, RiemannSum::Left, "Left");
                ui.radio_value(&mut self.riemann_sum_type, RiemannSum::Midpoint, "Midpoint");
                ui.radio_value(&mut self.riemann_sum_type, RiemannSum::Right, "Right");
                
            });

            if ui.button("calculate").clicked() {
                self.result = calculate_integral(&self.equ, &self.start, &self.end, self.riemann_sum_type, self.rectangle_count, &mut self.rectangle_heights);
            }

            ui.label(format!("{} square units", self.result));

            //user input buttons
            let number_buttons = ['x','0','1','2','3','4','5','6','7','8','9','.','+','-','*','/','^','(',')'];
            let trig_buttons = ["sin","cos","tan","asin","acos","atan"];
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP).with_main_wrap(true), |ui|{
                // Numbers and operators
                for i in 0..(number_buttons.len()) {
                    if ui.button(number_buttons[i].to_string()).clicked() {
                        self.equ.push(number_buttons[i])
                    }
                }
                // Trig functions
                for i in 0..(trig_buttons.len()) {
                    if ui.button(trig_buttons[i].to_string()).clicked() {
                        self.equ.push_str(trig_buttons[i])
                    }
                }

                //Backspace button
                if ui.button("Backspace").clicked() {
                    self.equ.pop();
                }

                //Clear button
                if ui.button("Clear").clicked() {
                    self.equ = "".to_string();
                }
            });

            ui.add(egui::Slider::new(&mut self.rectangle_count, 0..=1000000).text("Rectangles"));

            //Graph
            ui.label("Double tap to reset the graph");
            let line = Line::new(self.rectangle_heights.clone());
            Plot::new("g").show(ui, |plot_ui| plot_ui.line(line));
        });
    }
}

// The big calculation function.
// Probably shouldn't be called calculate_integral because it also calculates the graph
// Also handles multi-threading (only on native, not WASM)
fn calculate_integral(equ:&String, start:&String, end:&String, riemann_sum_type:RiemannSum, rectangles:i32, graph_points:&mut Vec<[f64;2]>) -> String{
    //Sanitizing really means removing spaces and formatting so that it's easier to work with
    let sanitized_equ = sanitize_equ(equ.clone());
    
    // Converts start and end to f64 or tells the user that they had an error
    let start:f64 = match start.parse() {
        Ok(start) => start,
        Err(_) => return "Error in start input".to_string(),
    };
    let end:f64 = match end.parse() {
        Ok(end) => end,
        Err(_) => return "Error in end input".to_string(),
    };
    

    //Multi-threaded integral calculations
    #[cfg(not(target_arch = "wasm32"))]
    let (area, mut points);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let thread_count = available_parallelism().unwrap_or(std::num::NonZeroUsize::new(1).unwrap()).get() as i32;
        let rectangles_per_thread = rectangles/thread_count;
        let threads_assigned_width = (end-start) / thread_count as f64;

        println!("Each thread is assigned width: {}", threads_assigned_width);

        let (sender, receiver) = unbounded();
        sender.send((0.0, vec![])).unwrap();

        thread::scope(|s|{
            for i in 0..(thread_count) {
                println!("{i}");
                s.spawn({
                    let sanitized_equ = sanitized_equ.clone();
                    let i = i.clone();
                    let sender = sender.clone();
                    let receiver = receiver.clone();
                    move ||{
                        let calculation = calculate_integral_single_thread(sanitized_equ, 
                            start + (i as f64 *threads_assigned_width), 
                            start + ((i as f64+1.)*threads_assigned_width), 
                            riemann_sum_type, 
                            rectangles_per_thread);


                        match calculation {
                            Some((area, mut points)) => {
                                let mut accumulated_volume = receiver.recv().unwrap();
                                accumulated_volume.0 += area;
                                accumulated_volume.1.append(&mut points);
                                sender.send(accumulated_volume).unwrap();
                            }
                            None => {
                                eprintln!("Error in thread");
                            }
                        }
                }});
            }
        });
        (area, points) = receiver.recv().unwrap();
    }
    #[cfg(target_arch = "wasm32")]
    let (area, mut points) = match calculate_integral_single_thread(sanitized_equ, start, end, riemann_sum_type, rectangles){
        Some(result) => result,
        None => return "Error in equation".to_string(),
    };

    graph_points.clear();
    graph_points.append(&mut points);
    graph_points.trim_to_x_points(10);
    graph_points.sort_by(|x,y| x[0].partial_cmp(&y[0]).unwrap());
    return area.to_string()
}


// Returns the integral of the subsection and the points (so that they can be graphed)
fn calculate_integral_single_thread(equ:String, start:f64,end:f64, riemann_sum_type:RiemannSum, rectangles:i32) -> Option<(f64, Vec<[f64;2]>)> {
    let rect_size = (end - start) / (rectangles as f64);
    let mut points = vec![];

    println!("calculating integral from {start} to {end} with {rectangles} rectangles of {rect_size} using a {riemann_sum_type} riemann sum");

    let mut rect_heights:Vec<f64> = vec![];

    for i in SimpleStepRange(start, end + rect_size, rect_size){
        match calculate::calculate(&equ.replace("x", &format!("({i})"))){
            Ok(result) => {
                rect_heights.push(result);
                points.push([i,result])
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    // If there's 0 or 1 rectangles, something is wrong, so just return None rather than causing weird errors later on
    if rect_heights.len() < 2 {
        return None;
    }

    let area:f64 = match riemann_sum_type {
        RiemannSum::Left => {
            rect_heights.remove(rect_heights.len()-1);
            let area = rect_heights.iter().sum::<f64>();
            area
        },
        RiemannSum::Right => {
            rect_heights.remove(0);
            let area = rect_heights.iter().sum::<f64>();
            area
        },
        RiemannSum::Midpoint => {
            //Midpoint is the same as trapezoid when rectangles are equal length
            //1/2(a + b)*w + (b + c)*w ...
            //w/2(a + 2b + 2c ... 2y + z)
            //w(1/2a + b + c ... + y + 1/2z)
            //(the w is used later)
            rect_heights[0] = rect_heights[0] * 0.5;

            let last_element = rect_heights.len()-1;
            rect_heights[last_element] = rect_heights[last_element] * 0.5;

            let area = rect_heights.iter().sum::<f64>();
            println!("area = {}", area);
            area
        },
    };

    println!("accumulated_volume is {}", area);

    Some((rect_size * area, points))
}

fn sanitize_equ(mut equ:String) -> String {
    //remove whitespace
    equ = equ.replace(" ",&"".to_string());
    equ = equ.replace("\n",&"".to_string());

    //make negatives not an operator
    equ = equ.replace("-",&"+-".to_string());
    equ = equ.replace("++",&"+".to_string());
    equ = equ.replace("^+-",&"^-".to_string());
    equ = equ.replace("(+-",&"(-".to_string());

    //make numbers before parentheses be multiplication
    equ = equ.replace("1(","1*(");
    equ = equ.replace("2(","2*(");
    equ = equ.replace("3(","3*(");
    equ = equ.replace("4(","4*(");
    equ = equ.replace("5(","5*(");
    equ = equ.replace("6(","6*(");
    equ = equ.replace("7(","7*(");
    equ = equ.replace("8(","8*(");
    equ = equ.replace("9(","9*(");
    equ = equ.replace("0(","0*(");

    // Same as above but with x
    equ = equ.replace("1x","1*x");
    equ = equ.replace("2x","2*x");
    equ = equ.replace("3x","3*x");
    equ = equ.replace("4x","4*x");
    equ = equ.replace("5x","5*x");
    equ = equ.replace("6x","6*x");
    equ = equ.replace("7x","7*x");
    equ = equ.replace("8x","8*x");
    equ = equ.replace("9x","9*x");
    equ = equ.replace("0x","0*x");

    return equ
}


struct SimpleStepRange(f64, f64, f64);  // start, end, and step

impl Iterator for SimpleStepRange {
    type Item = f64;

    #[inline]
    fn next(&mut self) -> Option<f64> {
        if self.0 < self.1 {
            let v = self.0;
            self.0 = v + self.2;
            Some(v)
        } else {
            None
        }
    }
}

trait TrimToXPoints {
    fn trim_to_x_points(&mut self, points:i32);
}

impl<T: Clone> TrimToXPoints for Vec<T>{
    fn trim_to_x_points(&mut self, points:i32) {
        let mut new_vec = vec![];
        for i in SimpleStepRange(0.0, self.len() as f64, (self.len() as i32 / points).into()) {
            let j = i as usize;
            new_vec.push(self[j].clone());
        }
    }
}