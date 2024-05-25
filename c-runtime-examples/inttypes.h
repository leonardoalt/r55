#ifndef _INTTYPES_H_
#define _INTTYPES_H_

/* Exact-width integer types */
typedef signed char int8_t;
typedef short int int16_t;
typedef int int32_t;
typedef long long int int64_t;

typedef unsigned char uint8_t;
typedef unsigned short int uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long long int uint64_t;

/* Minimum-width integer types */
typedef signed char int_least8_t;
typedef short int int_least16_t;
typedef int int_least32_t;
typedef long long int int_least64_t;

typedef unsigned char uint_least8_t;
typedef unsigned short int uint_least16_t;
typedef unsigned int uint_least32_t;
typedef unsigned long long int uint_least64_t;

/* Fastest minimum-width integer types */
typedef signed char int_fast8_t;
typedef int int_fast16_t;
typedef int int_fast32_t;
typedef long long int int_fast64_t;

typedef unsigned char uint_fast8_t;
typedef unsigned int uint_fast16_t;
typedef unsigned int uint_fast32_t;
typedef unsigned long long int uint_fast64_t;

/* Integer types capable of holding object pointers */
typedef int intptr_t;
typedef unsigned int uintptr_t;

/* Greatest-width integer types */
typedef long long int intmax_t;
typedef unsigned long long int uintmax_t;

#endif /* _INTTYPES_H_ */
