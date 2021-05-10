// This example demostrates the use of control statements

let a = 3;

// if statements
if(a == 3){
  print("a is 3");
}else if (a == 4){
  print("a is 4");
}else{
  print("a is something else");
}

// for loop
for(let i=0; i<20; i=i+1){
  // we can skip to the next iteration early using the continue statement
  if (i == 15){
    continue;
  }

  print("i is " + i);
}

exit();
