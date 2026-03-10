#include <stdio.h>
#include <pthread.h>
#include <unistd.h>

int shared_counter = 0;
pthread_mutex_t mutex = PTHREAD_MUTEX_INITIALIZER;

void *worker(void *arg) {
    int id = *(int *)arg;
    pthread_mutex_lock(&mutex);
    shared_counter++;
    printf("Thread %d: counter = %d\n", id, shared_counter);
    pthread_mutex_unlock(&mutex);
    return NULL;
}

int main(void) {
    pthread_t threads[3];
    int ids[3] = {1, 2, 3};

    for (int i = 0; i < 3; i++) {
        pthread_create(&threads[i], NULL, worker, &ids[i]);
    }
    for (int i = 0; i < 3; i++) {
        pthread_join(threads[i], NULL);
    }

    printf("Final counter: %d\n", shared_counter);
    return 0;
}
