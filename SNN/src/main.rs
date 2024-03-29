extern crate rand;
use crate::rand::Rng;
use std::io;

mod network;
mod neuron;
mod layer;
mod errors;

use network::Network;
use errors::{Type, ErrorComponent, ConfErr};
use neuron::Neuron;


fn main() {
    println!("Welcome to the Neural Network Configuration Menu!");
    let num_layers = get_input("\nEnter the number of layers: ");

    let mut network_conf = vec![0;num_layers];
    println!("\nNumber of neurons per layer: ");
    for i in 0..num_layers {
        let prompt = format!("-Layer {}: ", i);
        let num_neurons = get_input(&prompt);
        network_conf[i] = num_neurons as i32;
    }

    let mut network_test = Network::new_empty(network_conf.clone());

    let random_values: bool = get_yes_or_no("\nDo you want to generate random values for each neuron?");
    let random_weights: bool = get_yes_or_no("\nDo you want to generate random weights?");
    match random_values {
        true => { //random values
            match random_weights {
                true => { //random values
                    println!("\nGenereting network with random values and random weights");
                    // network_test = Network::new_random(network_conf);
                    network_test.add_random_neurons(lif);
                    network_test.add_random_weights();
                },
                false => { //by hand
                    println!("\nGenereting network with random values and configured weights");
                    network_test.add_random_neurons(lif);
                    network_test.add_weights_from_input();
                }
            }
        },
        false => { //by hand

            match random_weights {
                true => { //random values
                    println!("\nGenereting network with configured values and random weights");

                    network_test.add_neurons_from_input(lif);
                    network_test.add_random_weights();
                },
                false => { //by hand
                    println!("\nGenereting network with configured values and configured weights");

                    network_test.add_neurons_from_input(lif);
                    network_test.add_weights_from_input();
                }
            }
        }
    }
    network_test.print_network();

    let n_inputs = get_input("\nHow long should simulation lasts (in instant of time)?");
    let mut inputs = Vec::new();
    let random_inputs: bool = get_yes_or_no("\nDo you want random inputs?");
    match random_inputs {
        true => {
            for _ in 0..n_inputs{
                inputs.push(gen_inputs(network_test.network_conf[0] as usize));
            }
        },
        false => {
            for i in 0..n_inputs{
                println!("Filling inputs instant {}:", i);
                inputs.push(get_array_input_u8(network_test.network_conf[0] as usize));
            }
        }
    }

    let errors_flag: bool = get_yes_or_no("\nDo you want to add some errors?");
    let mut num_inferences = 0;
    let error_type;
    match errors_flag {
        true => {
            num_inferences = get_input("\nHow many inferences do you want?");
            error_type = get_error_type();
        },
        false => {
            println!("\nNo errors in the network");
            error_type = Type::None;
        }
    }

    let err_comp = get_error_component();
    println!("\n*********************************************\n");
    println!("Simulation without error: ");
    let outputs_no_err =  network_test.simulation_without_errors(inputs.clone());
    for j in 0..outputs_no_err.len(){
        println!("output {} : {:?}", j, outputs_no_err[j]);
    }
    println!("\n*********************************************\n");

    let mut count_err1 = 0;
    let mut count_err2 = 0;
    for i in 0..num_inferences{
        let error = ConfErr::new_from_main(&network_test, error_type, &err_comp ,n_inputs);
        println!("Simulation {}", i+1);
        println!("{}", error);

        let outputs =  network_test.simulation(inputs.clone(), error.clone());
        for j in 0..outputs.len(){
            println!("output {} : {:?}", j, outputs[j]);
        }
        let tmp = compute_differences1(&outputs_no_err, &outputs);
        count_err1 += tmp;
        count_err2 += compute_differences2(&outputs_no_err, &outputs);
        if tmp == 1{
            println!("\nERROR IN THIS SIMULATION!!!!");
        }
        println!("\n*********************************************\n");


    }
    println!("resilience1: {:.2}%, with errors: {}/{}", (num_inferences-count_err1)*100/num_inferences, count_err1,num_inferences);
    println!("resilience2: {:.2}%, with errors: {}/{}", (num_inferences*(outputs_no_err[0].len() * outputs_no_err.len())-count_err2)*100/(outputs_no_err[0].len() * outputs_no_err.len() * num_inferences), count_err2,(outputs_no_err[0].len() * outputs_no_err.len() * num_inferences));

}


pub fn lif(neuron :&mut Neuron, inputs_prec_layer: &Vec<u8>, inputs_same_layer: &Vec<u8>, error: &ConfErr, time: i32) -> u8{

    let mut flag_error = false;
    if error.id_neuron == neuron.id && ((error.err_type == Type::BitFlip && error.t_start == time) || (error.err_type == Type::Stuck0 || error.err_type == Type::Stuck1) ){
        flag_error = error.err_comp == ErrorComponent::Adder || error.err_comp == ErrorComponent::Multiplier;
        neuron.neuron_create_error(error);
    }

    let diff = neuron::adder(neuron.v_mem,  -neuron.v_rest, error, flag_error);
    let mul = neuron::multiplier(diff, f64::exp(-neuron.delta_t/0.1) , error, flag_error);
    neuron.v_mem = neuron::adder(neuron.v_rest, mul, error, flag_error);

    neuron.delta_t = 1.0;

    for i in 0..inputs_prec_layer.len(){
        let temp = neuron::multiplier(inputs_prec_layer[i] as f64, neuron.connections_prec_layer[i], error, flag_error);
        neuron.v_mem = neuron::adder(neuron.v_mem, temp, error,flag_error );
    }

    for i in 0..inputs_same_layer.len(){
        let temp = neuron::multiplier(inputs_same_layer[i] as f64, neuron.connections_same_layer[i], error, flag_error);
        neuron.v_mem = neuron::adder(neuron.v_mem, temp, error,flag_error);
    }

    if neuron.v_mem > neuron.v_threshold{
        neuron.v_mem = neuron.v_reset;
        return 1;
    }
    0
}

pub fn compute_differences1(right: &Vec<Vec<u8>>, output: &Vec<Vec<u8>>) -> usize{
    for i in 0..output.len(){
        for j in 0..output[i].len(){
            if right[i][j] != output[i][j]{
                return 1;
            }
        }
    }
    0
}

pub fn compute_differences2(right: &Vec<Vec<u8>>, output: &Vec<Vec<u8>>) -> usize{
    let mut count = 0;
    for i in 0..output.len(){
        for j in 0..output[i].len(){
            if right[i][j] != output[i][j]{
                count+=1;
            }
        }
    }
    count
}

pub fn gen_inputs( n_input: usize)-> Vec<u8>{
    let mut rnd = rand::thread_rng();
    let mut input = Vec::new();
    for _ in 0..n_input{
        input.push((rnd.gen_range(0..10) as u8)%2);
    }
    input
}

fn get_input(prompt: &str) -> usize {
    loop {
        println!("{}", prompt);

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim().parse() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please enter a valid number."),
        }
    }
}

fn get_input_f64(prompt: &str) -> f64 {
    loop {
        println!("{}", prompt);

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim().parse() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please enter a valid number."),
        }
    }
}

fn get_yes_or_no(prompt: &str) -> bool {
    loop {
        println!("{} (y/n)", prompt);

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Invalid input. Please enter 'y' for yes or 'n' for no."),
        }
    }
}

fn get_binary_input(prompt: &str) -> u8 {
    loop {
        println!("{} (1/0)", prompt);

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim() {
            "1" => return 1,
            "0" => return 0,
            _ => println!("Invalid input. Please enter 1 or 0 only"),
        }
    }
}

fn get_error_type() -> Type {
    println!("\nSelect the type of error:");
    println!("1. Stuck0");
    println!("2. Stuck1");
    println!("3. BitFlip");

    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim() {
            "1" => return Type::Stuck0,
            "2" => return Type::Stuck1,
            "3" => return Type::BitFlip,
            _ => println!("Invalid input. Please select a valid option (1, 2, or 3)."),
        }
    }
}

fn get_error_component() -> Vec<ErrorComponent> {
    let mut err_cmp_vec = Vec::new();
    println!("\nSelect error component for components list:");
    println!("1. Threshold");
    println!("2. VRest");
    println!("3. VMem");
    println!("4. VReset");
    println!("5. Weights");
    println!("6. Multiplier");
    println!("7. Adder");
    println!("8. Stop\n");

    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim() {
            "1" => err_cmp_vec.push(ErrorComponent::Threshold),
            "2" => err_cmp_vec.push(ErrorComponent::VRest),
            "3" => err_cmp_vec.push(ErrorComponent::VMem),
            "4" => err_cmp_vec.push(ErrorComponent::VReset),
            "5" => err_cmp_vec.push(ErrorComponent::Weights),
            "6" => err_cmp_vec.push(ErrorComponent::Multiplier),
            "7" => err_cmp_vec.push(ErrorComponent::Adder),
            "8" => return err_cmp_vec,
            _ => println!("Invalid input. Please select a valid option (1 ..= 8)."),
        }
    }
}

fn get_array_input(size: usize) -> Vec<f64> {
    let mut numbers = Vec::new();

    for i in 0..size {
        let number = get_input_f64(&format!("Enter number {}:", i + 1));
        numbers.push(number as f64);
    }

    numbers
}

fn get_array_input_u8(size: usize) -> Vec<u8> {
    let mut numbers = Vec::new();

    for i in 0..size {
        let number = get_binary_input(&format!("Enter input for neuron {} (first layer):", i + 1));
        numbers.push(number as u8);
    }

    numbers
}

