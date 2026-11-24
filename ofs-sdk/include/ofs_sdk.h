#ifndef OPEN_FRAME_STUDIO_SDK_V1_H
#define OPEN_FRAME_STUDIO_SDK_V1_H
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Native C alignment, x64. Returned by value. No memory to release. */
typedef struct {
    int32_t status; /* 0 = OK, 1 = invalid dimensions, 2 = internal error */
    double total_eur;
    double labor_hours;
    double glass_eur;
} OfsEstimateV1;

OfsEstimateV1 ofs_estimate_fixed_v1(double width_mm, double height_mm);

#ifdef __cplusplus
}
#endif
#endif
