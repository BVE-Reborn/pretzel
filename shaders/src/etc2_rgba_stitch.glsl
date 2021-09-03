// RGB and Alpha components of ETC2 RGBA are computed separately.
// This compute shader merely stitches them together to form the final result
// It's also used by RG11 driver to stitch two R11 into one RG11

#version 440

// #include "/media/matias/Datos/SyntaxHighlightingMisc.h"

#include "CrossPlatformSettings_piece_all.glsl"
#include "UavCrossPlatform_piece_all.glsl"

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform utexture2D srcRGB_split;
layout(set = 0, binding = 1) uniform utexture2D srcAlpha_split;
layout(set = 0, binding = 2) uniform sampler sampler_split;
#define srcRGB usampler2D(srcRGB_split, sampler_split)
#define srcAlpha usampler2D(srcAlpha_split, sampler_split)
layout(set = 0, binding = 3, rgba32ui) uniform restrict writeonly uimage2D dstTexture;

void main()
{
	uint2 etcRgb = OGRE_Load2D( srcRGB, int2( gl_GlobalInvocationID.xy ), 0 ).xy;
	uint2 etcAlpha = OGRE_Load2D( srcAlpha, int2( gl_GlobalInvocationID.xy ), 0 ).xy;

	imageStore( dstTexture, int2( gl_GlobalInvocationID.xy ), uint4( etcAlpha.xy, etcRgb.xy ) );
}
