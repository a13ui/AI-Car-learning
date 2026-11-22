use rand::RngExt;

use std::io::{BufRead, BufReader};
use std::fs::{self, File};

const MUTATION_RATE: f32 = 0.1;
const MUTATION_SCALE: f32 = 1.0;

pub struct Network {
    pub layers: Vec<Layer>,
    pub size: usize,

    pub memories: Vec<f32>,
    pub memory_size: usize,
    pub memory_function: Function,
}

pub struct Layer {
    pub neurons: Vec<Neuron>,
    pub size: usize,
    pub function: Function,
}

pub struct Neuron {
    pub bias: f32,
    pub weights: Vec<f32>,
    pub function: Function,

    pub size: usize,
}

#[derive(Clone)]
pub enum Function {
    Sigmoid,
    Relu,
    Tanh,
}

impl Network {
    pub fn new(sizes: Vec<usize>, function: Vec<Function>, memory_size: usize, memory_function: Function) -> Self {
        let size = sizes.len();
        Network { 
            layers: {
                let mut l  = Vec::new();
                for i in 1..size {
                    let mut prev_layer_size = sizes[i - 1];
                    if i == 1 {
                        prev_layer_size += memory_size;
                    }

                    let mut this_layer_size = sizes[i];
                    if i == size - 1 {
                        this_layer_size += memory_size
                    }

                    l.push(Layer::new(this_layer_size, prev_layer_size, &function[i - 1]));
                };
                l
            },
            size: size,
            memories: {
                let mut v  = Vec::new();
                for _i  in 0..memory_size {
                    v.push(0.0);
                };
                v
            }, 
            memory_size, 
            memory_function,
        }
    }
    pub fn save(&self, path: &str) {
        let mut output = String::new();
        output += &(self.size - 1).to_string();
        output += "\n";
        output += &self.memory_size.to_string();
        output += " ";
        output += match self.memory_function {
            Function::Sigmoid => "sigmoid",
            Function::Relu => "relu",
            Function::Tanh => "tanh",
        };
        output += "\n";
        for l in &self.layers {
            output += &l.save();
        }  
        let _ = fs::write(path, output);
    }
    pub fn load(path: &str) -> Self {
        let mut lines = BufReader::new(File::open(path).unwrap()).lines();

        let first_line = lines.next().unwrap().unwrap();
        let mut split_first_line = first_line.split(' ');

        let size = next_f32(&mut split_first_line) as usize;
        let memory_size = next_f32(&mut split_first_line) as usize;
        let memory_function = next_function(&mut split_first_line);

        let mut memories = Vec::new();
        for _i in 0.. memory_size {
            memories.push(0.0);
        }

        let mut layers = Vec::new();
        for _i in 0..size {
            layers.push(Layer::load(&mut lines));
        }
        Network { 
            layers,
            size,
            memories, 
            memory_size, 
            memory_function,
        }
    }
    pub fn run(&mut self, input: &Vec<f32>) -> Vec<f32> {
        let mut output = input.clone();

        for memory in &self.memories {
            output.push(run_function(&self.memory_function, memory.clone()));
        }

        for l in 0..self.layers.len(){
            output = self.layers[l].run(&output);
        };

        let memory_deltas = output.split_off(output.len() - self.memory_size);
        for m in 0..self.memories.len() {
            self.memories[m] += memory_deltas[m];
        }                                     

        output
    }
    pub fn vary(&self) -> Self {
        let mut layers = Vec::new();
        for l in 0..(self.size - 1) {
            layers.push(self.layers[l].vary());
        }
        Network {
            layers,
            size: self.size,
            memories: {
                let mut v  = Vec::new();
                for _i  in 0..self.memory_size{
                    v.push(0.0);
                };
                v
            }, 
            memory_size: self.memory_size, 
            memory_function: self.memory_function.clone(),         
        }
    }
    pub fn breed(a: &Self, b: &Self) -> Self {
        let mut layers = Vec::new();
        let size = a.size.min(b.size);
        for l in 0..(size - 1) {
            layers.push(Layer::breed(&a.layers[l], &b.layers[l]));
        }
        Network {
            layers,
            size, 
            memories: {
                let mut v  = Vec::new();
                for _i  in 0..a.memory_size {
                    v.push(0.0);
                };
                v
            }, 
            memory_size: a.memory_size, 
            memory_function: a.memory_function.clone(),         
        }
    }
}

impl Layer {
    fn new(size: usize, prev_layer_size: usize, function: &Function) -> Self {
        Layer {
            neurons: {
                let mut n = Vec::new();
                for _i in 0..size {
                    n.push(Neuron::new(prev_layer_size, function.clone()));
                };
                n
            },
            size: size,
            function: function.clone(),
        }
    }
    fn save(&self) -> String {
        let mut output = String::from("\n");
        output += &self.size.to_string();
        output += " ";
        output += match self.function {
            Function::Sigmoid => "sigmoid",
            Function::Relu => "relu",
            Function::Tanh => "tanh",
        };
        for n in &self.neurons {
            output += &n.save();
        }
        output
    }
    fn load(lines: &mut std::io::Lines<BufReader<File>>) -> Self {
        let first_line = lines.next().unwrap().unwrap();
        let mut split_first_line = first_line.split(' ');

        let size = next_f32(&mut split_first_line) as usize;
        let function = next_function(&mut split_first_line);

        let mut neurons = Vec::new();
        for _i in 0..size {
            neurons.push(Neuron::load(lines.next().unwrap().unwrap(), function.clone()));
        }
        Layer {
            neurons,
            size,
            function: function.clone(),
        }
    }
    fn run(&self, input: &Vec<f32>) -> Vec<f32>{
        let mut output = Vec::new();
        for n in 0..self.neurons.len() {
            output.push(self.neurons[n].run(&input));
        };
        output
    }
    fn vary(&self) -> Self {
        let mut neurons = Vec::new();
        for n in 0..self.size {
            neurons.push(self.neurons[n].vary());
        }
        Layer {
            neurons,
            size: self.size,
            function: self.function.clone(),
        }
    }
    fn breed(a: &Self, b: &Self) -> Self {
        let mut neurons = Vec::new();
        for i in 0..a.size.min(b.size) {
            neurons.push(Neuron::breed(&a.neurons[i], &b.neurons[i]));
        }
        Layer {
            neurons,
            size: a.size,
            function: a.function.clone(),
        }
    }
}

impl Neuron {
    fn new(prev_layer_size: usize, func: Function) -> Self {    
        Neuron { 
            bias: ran_param(), 
            weights: {
                let mut w = Vec::new();
                for _i in 0..prev_layer_size {
                    w.push(ran_param());
                };
                w
            },
            size: prev_layer_size,
            function: func,
        }
    }
    fn save(&self) -> String {
        let mut output = String::from("\n");
        output += &self.bias.to_string();
        for weight in &self.weights {
            output += " ";
            output += &weight.to_string();
        }
        output
    }
    fn load(line: String, function: Function) -> Self {
        let mut line_words = line.split(' ');
        let bias = next_f32(&mut line_words);
        let mut weights = Vec::new();
        for word in line_words {
            weights.push(parse_f32(word));
        }
        let size = weights.len();
        Neuron { 
            bias,
            weights, 
            function, 
            size,
        }
    }
    fn run(&self, input: &Vec<f32>) -> f32 {
        let mut output = self.bias;
        for w in 0..self.weights.len() {
            output += input[w] * self.weights[w];
        };
        run_function(&self.function, output)
    }
    fn vary(&self) -> Self {
        let mut weights = Vec::new();
        for i in 0..self.size {
            weights.push(vary(self.weights[i]));
        }
        Neuron {
            bias: vary(self.bias),
            weights,
            size: self.size,
            function: self.function.clone(),
        }
    }
    fn breed(a: &Self, b: &Self) -> Self {
        let mut weights = Vec::new();
        for i in 0..a.size.min(b.size) {
            weights.push(crossover(a.weights[i], b.weights[i]));
        }
        Neuron {
            bias: crossover(a.bias, b.bias),
            weights,
            size: a.size,
            function: a.function.clone(),
        }
    }
}

fn run_function(function: &Function, x: f32) -> f32 {
    match function {
        Function::Sigmoid => sigmoid(x),
        Function::Relu => relu(x),
        Function::Tanh => tanh(x),
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 - (1.0 / (1.0 + x.exp()))
}
fn relu(x: f32) -> f32 {
    x.max(0.0)
}
fn tanh(x: f32) -> f32 {
    x.tanh()
}

fn vary(a: f32) -> f32 {
    let mut rng = rand::rng();
    if rng.random::<f32>() < MUTATION_RATE {
        a + rng.random_range(-MUTATION_SCALE..MUTATION_SCALE)
    } else {
        a
    }
}
fn crossover(a: f32, b: f32) -> f32 {
    let mut rng = rand::rng();
    let pick = if rng.random::<bool>() { a } else { b };
    if rng.random::<f32>() < MUTATION_RATE {
        pick + rng.random_range(-MUTATION_SCALE..MUTATION_SCALE)
    } else {
        pick
    }
}
fn ran_param() -> f32 {
    let mut rng = rand::rng();
    rng.random::<f32>() * 2.0 - 1.0
}

fn next_f32(iter: &mut std::str::Split<'_, char>) -> f32 {
    parse_f32(iter.next().unwrap())
}
fn next_function(iter: &mut std::str::Split<'_, char>)-> Function {
    match iter.next().unwrap() {
        "sigmoid" => Function::Sigmoid,
        "relu" => Function::Relu,
        "tanh" => Function::Tanh,
        _ => panic!("unknown function type"),
    }
}
fn parse_f32(string: &str) -> f32 {
    string.parse::<f32>().unwrap()
}