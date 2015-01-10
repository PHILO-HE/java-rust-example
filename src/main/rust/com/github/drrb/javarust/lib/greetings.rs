/*
 * Copyright (C) 2015 drrb
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

// Create a library, not an executable binary
#![crate_type = "dylib"]

extern crate libc;

use std::ffi::CString;
use std::ffi;
use std::str;
use std::mem;
use libc::{c_int,c_char};

// GreetingSet corresponds to com.github.drrb.javarust.GreetingSet in Java. It is marked with
// repr(c), as are all the structs passed back to Java. This makes sure the structs are represented
// in memory in a way JNA can read them.
#[repr(C)]
pub struct GreetingSet {
    // A pointer to an array of Greetings. This is converted to a Greeting.ByReference by JNA.
    greetings: Box<[Greeting]>,
    // The size of the array. We need to pass it back to Java so that we know how long the array
    // is (JNA can't guess the size). We need to do this with all arrays created in Java and read
    // in Rust (or vise-versa). This c_int is converted to a Java int by JNA.
    number_of_greetings: c_int
}

// Greeting corresponds to com.github.drrb.javarust.Greeting in Java. It is marked with
// allow(missing_copy_implementations) to suppress compiler warnings encouraging us to 
// implement the Copy trait.
#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct Greeting {
    // A pointer to the beginning of a string. Converted to a Java String by JNA. All strings
    // passed between Rust and Java are represented in this way.
    text: *const c_char
}

impl Greeting {
    // A constructor, for convenience
    fn new(string: &str) -> Greeting {
        Greeting { text: to_ptr(string.to_string()) }
    }
}

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct Person {
    first_name: *const c_char,
    last_name: *const c_char
}

/// Example of just calling into Rust
/// It is marked as "no_mangle", so that our Java code can still see the Rust function after it's
/// compiled (normally the Rust compiler changes the name during compilation. It is marked as
/// allow(snake_case) because Rust functions are supposed to be written in snake_case, but we need
/// to use camelCase to match the name of the function in Java.
#[no_mangle]
#[allow(non_snake_case)]
pub extern fn printGreeting(name: *const c_char) {
    // Convert the C string to a Rust one
    let name = to_string(&name);
    println!("Hello, {}", name);
}

/// Example of passing and returning a value
/// The string argument and return types are native C strings (pointers to arrays of c_chars).
#[no_mangle]
#[allow(non_snake_case)]
pub extern fn renderGreeting(name: *const c_char) -> *const c_char {
    let name = to_string(&name);

    // Convert the Rust string back to a C string so that we can return it
    to_ptr(format!("Hello, {}!", name))
}

/// Example of passing a struct to Rust
#[no_mangle]
pub extern fn greet(person: &Person) -> *const c_char {
    let first_name = to_string(&person.first_name);
    let last_name = to_string(&person.last_name);
    to_ptr(format!("Hello, {} {}!", first_name, last_name))
}

/// Example of returning a struct from Rust by value
#[no_mangle]
#[allow(non_snake_case)]
pub extern fn getGreetingByValue() -> Greeting {
    Greeting::new("Hello from Rust!")
}

/// Example of returning a struct from Rust by reference
/// Note that we return an owned pointer to the struct (i.e a Box containing the struct). This
/// tells Rust that the Java code now "owns" the pointer, so Rust shouldn't try to clean it up at
/// the end of the function.
#[no_mangle]
#[allow(non_snake_case)]
pub extern fn getGreetingByReference() -> Box<Greeting> {
    Box::new(Greeting::new("Hello from Rust!"))
}

/// Example of passing a callback function
/// The callback from Java (an object with an apply() method) is turned into a function pointer by
/// JNA.
#[no_mangle]
#[cfg(not(windows))]
#[allow(non_snake_case)]
pub extern fn callMeBack(callback: extern "C" fn(*const c_char)) { // The function argument here is an "extern" one, so that we can pass it in from Java
    // Call the Java method
    callback(to_ptr("Hello there!".to_string()));
}

/// Example of passing a callback (Windows version)
/// Note that the callback version is marked as "stdcall", because that is the calling convention
/// Windows uses.
#[no_mangle]
#[cfg(windows)]
#[allow(non_snake_case)]
pub extern fn callMeBack(callback: extern "stdcall" fn(*const c_char)) {
    callback(to_ptr("Hello there!".to_string()));
}

/// More complicated callback example
/// In this example we send a pointer to a struct back to Java via the callback.
#[no_mangle]
#[allow(non_snake_case)]
pub extern fn sendGreetings(callback: extern "C" fn(Box<GreetingSet>)) {
    let greetings = vec![ Greeting::new("Hello!"), Greeting::new("Hello again!") ];
    let num_greetings = greetings.len();

    let set = Box::new(GreetingSet {
        // Get a pointer to the vector as an array, so that we can pass it back to Java
        greetings: greetings.into_boxed_slice(),
        // Also return the length of the array, so that we can create the array back in Java
        number_of_greetings: num_greetings as c_int
    });
    callback(set);
}

/// Example of returning a more complicated struct from Rust
#[no_mangle]
#[allow(non_snake_case)]
pub extern fn renderGreetings() -> Box<GreetingSet> {
    let greetings = vec![ Greeting::new("Hello!"), Greeting::new("Hello again!") ];
    let num_greetings = greetings.len();

    Box::new(GreetingSet {
        greetings: greetings.into_boxed_slice(),
        number_of_greetings: num_greetings as c_int
    })
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern fn dropGreeting(_: Box<Greeting>) {
    // Do nothing here. Because we own the Greeting here (we're using a Box) and we're not
    // returning it, Rust will assume we don't want it anymore and clean it up.
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern fn dropGreetingSet(_: Box<GreetingSet>) {
    // Do nothing here. Because we own the GreetingSet here (we're using a Box) and we're not
    // returning it, Rust will assume we don't want it anymore and clean it up.
}

/// Convert a native string to a Rust string
fn to_string(pointer: &*const c_char) -> String {
    let slice = unsafe { ffi::c_str_to_bytes(pointer) };
    str::from_utf8(slice).unwrap().to_string()
}

/// Convert a Rust string to a native string
fn to_ptr(string: String) -> *const c_char {
    let cs = CString::from_slice(string.as_bytes());
    let ptr = cs.as_ptr();
    // Tell Rust not to clean up the string while we still have a pointer to it.
    // Otherwise, we'll get a segfault.
    unsafe { mem::forget(cs) };
    ptr
}
