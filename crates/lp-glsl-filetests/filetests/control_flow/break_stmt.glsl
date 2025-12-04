// test compile
// test run

int main() {
    int sum = 0;
    int i = 0;
    while (i < 100) {
        if (i == 5) {
            break;
        }
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}

// run: == 10


