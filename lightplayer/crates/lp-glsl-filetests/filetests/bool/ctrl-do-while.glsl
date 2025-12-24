// test run
// target riscv32.fixed32

// ============================================================================
// Control flow: do { } while (bool) - condition must evaluate to bool
// ============================================================================

int test_do_while_bool_true() {
    bool b = true;
    int x = 0;
    int count = 0;
    do {
        x = x + 1;
        count = count + 1;
        if (count >= 3) {
            b = false;  // Exit after 3 iterations
        }
    } while (b);
    return x;
    // Should be 3 (loop runs at least once, then 3 times total)
}

// run: test_do_while_bool_true() == 3

int test_do_while_bool_false() {
    bool b = false;
    int x = 0;
    do {
        x = x + 1;
    } while (b);
    return x;
    // Should be 1 (do-while runs at least once even if condition is false)
}

// run: test_do_while_bool_false() == 1

int test_do_while_bool_runs_once() {
    bool b = false;
    int x = 0;
    do {
        x = 5;
    } while (b);
    return x;
    // Should be 5 (runs once even though condition is false)
}

// run: test_do_while_bool_runs_once() == 5

int test_do_while_bool_expression() {
    int x = 0;
    int i = 0;
    do {
        x = x + 1;
        i = i + 1;
    } while (i < 4);  // Comparison returns bool
    return x;
    // Should be 4 (loop runs 4 times)
}

// run: test_do_while_bool_expression() == 4

int test_do_while_bool_change_condition() {
    bool b = true;
    int x = 0;
    int count = 0;
    do {
        x = x + 1;
        count = count + 1;
        if (count >= 5) {
            b = false;  // Exit after 5 iterations
        }
    } while (b);
    return x;
    // Should be 5 (loop runs 5 times)
}

// run: test_do_while_bool_change_condition() == 5

int test_do_while_bool_not() {
    bool done = false;
    int x = 0;
    int count = 0;
    do {
        x = x + 1;
        count = count + 1;
        if (count >= 2) {
            done = true;
        }
    } while (!done);
    return x;
    // Should be 2 (loop runs 2 times: !false=true, then !true=false)
}

// run: test_do_while_bool_not() == 2

int test_do_while_bool_complex() {
    bool a = true;
    bool b = true;
    int x = 0;
    int count = 0;
    do {
        x = x + 1;
        count = count + 1;
        if (count >= 3) {
            a = false;
        }
    } while (a && b);
    return x;
    // Should be 3 (loop runs 3 times: a&&b=true, then a=false so exits)
}

// run: test_do_while_bool_complex() == 3

