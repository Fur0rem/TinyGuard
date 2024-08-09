#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct {
	size_t size;
	size_t capacity;
	int* data;
} IntVector;

IntVector IntVector_new() {
	IntVector v = (IntVector){.size = 0, .capacity = 0, .data = NULL};
	return v;
}

void IntVector_push_back(IntVector* v, int value) {
	if (v->size == v->capacity) {
		size_t new_capacity = v->capacity == 0 ? 1 : v->capacity * 2;
		int* new_data = (int*)malloc(new_capacity * sizeof(int));
		for (size_t i = 0; i < v->size; i++) {
			new_data[i] = v->data[i];
		}
		free(v->data);
		v->data = new_data;
		v->capacity = new_capacity;
	}
	v->data[v->size] = value;
	v->size++;
}

void IntVector_free(IntVector* v) {
	free(v->data);
	v->size = 0;
	v->capacity = 0;
	v->data = NULL;
}

int compare_ints(const void* a, const void* b) {
	return *(const int*)a - *(const int*)b;
}

void IntVector_sort(IntVector* v) {
	qsort(v->data, v->size, sizeof(int), compare_ints);
}

size_t IntVector_search(const IntVector* v, int value) {
	for (size_t i = 0; i < v->size; i++) {
		if (v->data[i] == value) {
			return i;
		}
	}
	return v->size;
}

size_t IntVector_search_sorted(const IntVector* v, int value) {
	size_t left = 0;
	size_t right = v->size;
	while (left < right) {
		size_t middle = left + (right - left) / 2;
		if (v->data[middle] < value) {
			left = middle + 1;
		}
		else {
			right = middle;
		}
	}
	return left;
}

int main() {
	IntVector vec = IntVector_new();
	IntVector_push_back(&vec, 3);
	IntVector_push_back(&vec, 1);
	IntVector_push_back(&vec, 4);
	IntVector_push_back(&vec, 1);

	size_t index = IntVector_search_sorted(&vec, 1);
	IntVector_sort(&vec);
	size_t index2 = IntVector_search(&vec, 1);

	printf("Index: %zu, Index2: %zu\n", index, index2);

	IntVector vec2;
	vec2 = IntVector_new();
	return 0;
}