
	.386
	.model FLAT

	.code

check_sample_or_compteur:
	cmp dx, 107h ; EDU_VoiceLine
	jz locret_samplecompteur

	cmp dx, 10Ah ; MS_compteur
locret_samplecompteur:
	retn

load_correct_hitpoints:
	xor eax, eax

	cmp dx, 107h ; EDU_Voiceline
	jnz singlebyte_hitpoints

	mov ax, [ebx+50h] ; EDU_ExtendedHitPoints
	jmp locret_hitpoints

singlebyte_hitpoints:
	mov al, [ebx+7Bh] ; InitialHitPoints

locret_hitpoints:
	retn
