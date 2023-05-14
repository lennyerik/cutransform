extern int threadIdxX(void);

void kernel(int *arr) {
    arr[threadIdxX()] = 123;
}

