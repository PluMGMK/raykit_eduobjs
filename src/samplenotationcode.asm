
	.386
	.model FLAT

	.code

replacement_vsprintf:
	cmp dx, 107h ; EDU_VoiceLine
	jnz vsprintf

	movzx eax, word ptr [ebx+50h] ; EDU_ExtHitPoints
	call GET_SAMPLE_NAME

	mov dword ptr [esp+12], eax ; Replace the third argument to vsprintf with the sample name
	jmp vsprintf

GET_SAMPLE_NAME:
	; This stub label is here to make it assemble
	; The Rust code will replace its address with that of the real GET_SAMPLE_NAME function.
	retn

; 128 bytes of nop to make sure the above jumps are long and can be substituted efficiently from Rust code!
db 128 dup(90h)

vsprintf:
	; This stub label is here to make it assemble
	; The Rust code will replace its address with that of the real vsprintf function.
	retn
