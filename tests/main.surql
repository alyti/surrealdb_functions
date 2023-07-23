/* Returns a greeting */
DEFINE FUNCTION fn::greet_but_with_number($name: string, $number: number) {
    RETURN "Hello " + $name + ", here's a number you gave me just now: " + <string> $number + "!";
};

/* Returns a greeting but its nested this time! */
DEFINE FUNCTION fn::nested::greet($name: string) {
    RETURN "Hello, " + $name + "!";
};
