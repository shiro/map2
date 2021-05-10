// This example demonstrates the ues of functions

// built-in functions can be called directly
print("hello world");

// custom functions are defined as variables
let my_function = ||{
  print("hello from my_function");
};

// custom functions are called the same way as built-ins
my_function();

// a function can accept parameters and return values

let sum = |a, b|{
  return a + b;
};

print("1 + 2 = " + sum(1, 2));

exit();
