/* Returns a greeting */
DEFINE FUNCTION fn::greet_but_with_number($name: string, $number: number) {
    RETURN "Hello, " + $name + " also here's a number: " + <string> $number + "!";
};

/* Returns a greeting but its nested this time! */
DEFINE FUNCTION fn::nested::greet($name: string) {
    RETURN "Hello, " + $name + "!";
};
