
	.386
	.model FLAT

	.code

check_freezable:
	cmp dx, 103h ; TYPE_EDU_LETTRE
	jz check_left
	cmp dx, 104h ; TYPE_EDU_CHIFFRE
	jz check_left
	cmp dx, 112h ; Unk10
	jz check_left
	cmp dx, 106h ; EDU_ArtworkObject
	jnz check_sample

check_left:
	cmp bl, 4Bh
	jnz check_right

	movzx eax, byte ptr [ecx+6Eh] ; RuntimeCurrentAnimIndex
	imul eax, 0Ch ; size R1_PS1_AnimationDescriptor
	add eax, [ecx+4] ; AnimDescriptorsPointer
	movzx esi, word ptr [eax+0Ah] ; FrameCount

	movzx eax, byte ptr [ecx+6Fh] ; RuntimeCurrentAnimFrame
	add eax, esi
	dec eax ; CurrentAnimFrame += FrameCount - 1

	jmp modulo

check_right:
	cmp bl, 4Dh
	jnz check_sample

	movzx eax, byte ptr [ecx+6Eh] ; RuntimeCurrentAnimIndex
	imul eax, 0Ch ; size R1_PS1_AnimationDescriptor
	add eax, [ecx+4] ; AnimDescriptorsPointer
	movzx esi, word ptr [eax+0Ah] ; FrameCount

	movzx eax, byte ptr [ecx+6Fh] ; RuntimeCurrentAnimFrame
	inc eax ; CurrentAnimFrame ++

modulo:
	mov edi, edx ; Store the value of edx since idiv spoils that register
	xor edx, edx
	idiv esi
	mov [ecx+6Fh], dl ; RuntimeCurrentAnimFrame = CurrentAnimFrame % FrameCount

	; If it's an artwork, we also need to update the hitpoints accordingly.
	cmp di, 106h ; EDU_ArtworkObject
	jnz restore_edx

	mov [ecx+7Ah], dl ; HitPoints
	mov [ecx+7Bh], dl ; InitialHitPoints
	jmp restore_edx

check_sample:
	cmp dx, 107h ; EDU_VoiceLine
	jnz locret

	movzx eax, word ptr [ecx+50h] ; EDU_ExtHitPoints
	cmp bl, 4Bh
	jnz check_right_sample

	add ax, word ptr ds:0BA88h ; num_samples (assuming the fixups work...)
	dec ax
	jmp modulo_sample

check_right_sample:
	cmp bl, 4Dh
	jnz check_down_sample

	inc ax
	jmp modulo_sample

check_down_sample:
	cmp word ptr ds:0BA88h, 50
	; If there are fewer than 50 samples, don't bother jump-scrolling
	jle locret

	cmp bl, 50h
	jnz check_up_sample

	add ax, word ptr ds:0BA88h ; num_samples
	sub ax, 50 ; decimal 50!
	jmp modulo_sample

check_up_sample:
	cmp bl, 48h
	jnz locret

	add ax, 50 ; decimal 50!
	jmp modulo_sample

modulo_sample:
	movzx esi, word ptr ds:0BA88h ; num_samples
	mov edi, edx ; Store the value of edx since idiv spoils that register
	xor edx, edx
	idiv esi
	mov [ecx+50h], dx

restore_edx:
	mov edx, edi

locret:
	; This CMP instruction is what we are replacing with a call to this custom function.
	; In the patcher as currently written, it will in turn be replaced with the colour-check function!
	cmp dx, 110h ; MS_pap
	retn
