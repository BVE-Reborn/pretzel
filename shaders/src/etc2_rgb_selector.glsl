#version 430 core

// RGB and Alpha components of ETC2 RGBA are computed separately.
//
// ETC2 also adds modes T and H (which we compute together) and mode P
//
// This shader will:
//	1. Select the mode with the lowest error
//	2. If not using Alpha, it will output to where P mode was stored (to save VRAM)
//	3. If using Alpha, it will also stitch the alpha and output to another texture
//
// See etc2_rgba_selector.glsl for this shader performing stitching (defines HAS_ALPHA)

// #include "/media/matias/Datos/SyntaxHighlightingMisc.h"

#include "CrossPlatformSettings_piece_all.glsl"
#include "UavCrossPlatform_piece_all.glsl"

layout(local_size_x = 8, local_size_y = 8) in;

layout(set = 0, binding = 0) uniform sampler sampler_split;
layout(set = 0, binding = 1) uniform texture2D texErrorEtc1_split;
layout(set = 0, binding = 2) uniform texture2D texErrorTh_split;
layout(set = 0, binding = 3) uniform texture2D texErrorP_split;

layout(set = 0, binding = 4) uniform utexture2D srcEtc1_split;
layout(set = 0, binding = 5) uniform utexture2D srcThModes_split;

#define texErrorEtc1 sampler2D(texErrorEtc1_split, sampler_split)
#define texErrorTh sampler2D(texErrorTh_split, sampler_split)
#define texErrorP sampler2D(texErrorP_split, sampler_split)
#define srcEtc1 usampler2D(srcEtc1_split, sampler_split)
#define srcThModes usampler2D(srcThModes_split, sampler_split)

#ifdef HAS_ALPHA
layout(set = 0, binding = 6 ) uniform utexture2D srcPMode_split;
layout(set = 0, binding = 7 ) uniform utexture2D srcAlpha_split;
#define srcPMode usampler2D(srcPMode_split, sampler_split)
#define srcAlpha usampler2D(srcAlpha_split, sampler_split)

layout(set = 0, binding = 8, rgba32ui ) uniform restrict writeonly uimage2D dstTexture;
#	define outputValue uint4( etcAlpha.xy, etcRgb.xy )
#else
layout(set = 0, binding = 8, rg32ui) uniform restrict uimage2D inOutPMode;
#	define srcPMode inOutPMode
#	define dstTexture inOutPMode
#	define outputValue uint4( etcRgb.xy, 0u, 0u )
#endif

void main()
{
	const float errorEtc1 = OGRE_Load2D( texErrorEtc1, int2( gl_GlobalInvocationID.xy ), 0 ).x;
	const float errorThModes = OGRE_Load2D( texErrorTh, int2( gl_GlobalInvocationID.xy ), 0 ).x;
	const float errorPMode = OGRE_Load2D( texErrorP, int2( gl_GlobalInvocationID.xy ), 0 ).x;

#ifdef HAS_ALPHA
	const uint2 etcAlpha = OGRE_Load2D( srcAlpha, int2( gl_GlobalInvocationID.xy ), 0 ).xy;
#endif

	if( errorEtc1 <= errorThModes && errorEtc1 <= errorPMode )
	{
		const uint2 etcRgb = OGRE_Load2D( srcEtc1, int2( gl_GlobalInvocationID.xy ), 0 ).xy;
		imageStore( dstTexture, int2( gl_GlobalInvocationID.xy ), outputValue );
	}
	else if( errorThModes < errorPMode )
	{
		const uint2 etcRgb = OGRE_Load2D( srcThModes, int2( gl_GlobalInvocationID.xy ), 0 ).xy;
		imageStore( dstTexture, int2( gl_GlobalInvocationID.xy ), outputValue );
	}
	else
	{
#ifdef HAS_ALPHA
		const uint2 etcRgb = OGRE_Load2D( srcPMode, int2( gl_GlobalInvocationID.xy ), 0 ).xy;
#else
		const uint2 etcRgb = OGRE_imageLoad2D( srcPMode, int2( gl_GlobalInvocationID.xy ) ).xy;
#endif
		imageStore( dstTexture, int2( gl_GlobalInvocationID.xy ), outputValue );
	}
}
