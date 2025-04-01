

input a brainfuck program till null character
-v>
,
[>,]
^-<+[-<+]-

need to translate to small numbers for easier coding later
>
+[-
	v------ ------ ------ ------ ------ ------ ------^
>+]-<+[-<+]-



plus	1	1
minus	3	2
left	18	4
right	20	3

openb	49	5
closeb	51	6

dot		4	7
comma	2	8

other	0


for each character in the bf file convert the char to a digit as above
>+[-v
copy n up two spaces
[-^+^+vv]^[-v+^]^

if n == 1:
-
>+<[>-<
else n != 1

	if n == 2:
	-
	>+<[>-<
	else n != 2

		if n == 3:
		-
		>+<[>-<
		else n != 3

			if n == 4:
			-
			>+<[>-<
			else n != 4

				if n == 18:
				------- -------
				>+<[>-<
				else n != 18

					if n == 20:
					--
					>+<[>-<
					else n != 20

						if n == 49:
						----- ----- ----- ----- --- --- ---
						>+<[>-<
						else n != 49

							if n == 51:
							--
							>+<[>-<
							else n != 51

								vv
								[-]
								^^

							[-]]>[[-]<[-]
							if n == 51 (closeb)
								vv
								[-]++++++
								v++^
								^^
							>]<

						[-]]>[[-]<[-]
						if n == 49 (openb)
							vv
							[-]+++++
							v+^
							^^
						>]<

					[-]]>[[-]<[-]
					if n == 20 (right)
						vv
						[-]+++
						^^
					>]<

				[-]]>[[-]<[-]
				if n == 18 (left)
					vv
					[-]++++
					^^
				>]<

			[-]]>[[-]<[-]
			if n == 4 (dot)
				vv
				[-]+++++++
				^^
			>]<

		[-]]>[[-]<[-]
		if n == 3 (minus)
			vv
			[-]++
			^^
		>]<

	[-]]>[[-]<[-]
	if n == 2 (comma)
		vv
		[-]++++++++
		^^
	>]<

[-]]>[[-]<[-]
if n == 1 (plus)
	vv
	[-]+
	^^
>]<vv

^>+]-


add boundaries
v-v-v-v-^^^^<+[-<+]<->v-v-v<-v->^^^^>

tape head
vvv+^^^
pc
<+>

main brainfuck loop:

+[-
<->+

	copy the current command to a clean space
	v[-^^+^+vvv]^^[-vv+^^]^

	if else chain:
	>+<-[>[-]<
	else not plus
		>+<-[>[-]<
		else not minus
			>+<-[>[-]<
			else not right
				>+<-[>[-]<
				else not left
					>+<-[>[-]<
					else not openb
						>+<-[>[-]<
						else not closeb
							>+<-[>[-]<
							else not dot
								>+<-[>[-]<[-]
								else not comma
									non brainfuck character
									do nothing
								]>[-<
								if comma
									vvvvv+[-<+]->-[+>-]+

									v,^

									+[-<+]-<^^^+[->+]->-[+>-]+^^
								>]<
							]>[-<
							if dot
								vvvvv+[-<+]->-[+>-]+

								v.^

								+[-<+]-<^^^+[->+]->-[+>-]+^^
							>]<
						]>[-<
						if closeb
							find the current cell
							vvvvv+[-<+]->-[+>-]+

							v[-v+v+^^]v[-^+v]v

							[[-]
							if tape cell is not 0 go through the loops and find the right one to skip back to
							starts 2 below the tape
								
								^^^+[-<+]-^^^+[->+]-
								set the loop counter to 1
								<+

								[>>-[+>-]<+vv
								go through each loop marker until the counter is 0
									copy to above for if else
									[-^^^+^+vvvv]^^^[-vvv+^^^]^

									->+<[>-<
									else not loop start
										->+<[>-<[-]
										else neither loop start or end
											pass
										]>[-<
										if loop end take 1 from counter
											find counter
											vv+[-<+]-<
											+
											navigate back
											>>-[+>-]+^^
										>]<
									]>[-<
									if loop start add 1 to counter
										vv+[-<+]-<
										-
										>>-[+>-]+^^
									>]<
								vv+[-<+]-<]

								clear loop counter just in case
								[-]

								return back to 2 below the tape
								>vvv+[-<+]->-[+>-]+vvv
							]

							^^^+[-<+]-<^^^+[->+]->-[+>-]+^^
						>]<
					]>[-<
					if openb
						find the current cell
						vvvvv+[-<+]->-[+>-]+

						v[-v+v+^^]v[-^+v]v

						>+<[>-<[-]]>[-<
						if tape cell is 0 go through the loops and find the right one to skip to
						starts 2 below the tape
							
							^^^+[-<+]-^^^+[->+]-
							set the loop counter to 1
							<+

							[>>-[+>-]>+vv
							go through each loop marker until the counter is 0
								copy to above for if else
								[-^^^+^+vvvv]^^^[-vvv+^^^]^

								->+<[>-<
								else not loop start
									->+<[>-<[-]
									else neither loop start or end
										pass
									]>[-<
									if loop end take 1 from counter
										find counter
										vv+[-<+]-<
										-
										navigate back
										>>-[+>-]+^^
									>]<
								]>[-<
								if loop start add 1 to counter
									vv+[-<+]-<
									+
									>>-[+>-]+^^
								>]<
							vv+[-<+]-<]

							clear loop counter just in case
							[-]

							return back to 2 below the tape
							>vvv+[-<+]->-[+>-]+vvv
						>]<

						^^^+[-<+]-<^^^+[->+]->-[+>-]+^^
					>]<
				]>[-<
				if left
					vvvvv+[-<+]->-[+>-]+

					check for and extend boundary if needed
					<<-v-^>+v+^
					[<+v+^>-v-^]
					+>-

					+[-<+]-<^^^+[->+]->-[+>-]+^^
				>]<
			]>[-<
			if right
				vvvvv+[-<+]->-[+>-]+
				
				>>-v-^<+v+^
				[>+v+^<-v-^]
				+<-

				+[-<+]-<^^^+[->+]->-[+>-]+^^
			>]<
		]>[-<
		if minus
			vvvvv+[-<+]->-[+>-]+

			v-^

			+[-<+]-<^^^+[->+]->-[+>-]+^^
		>]<
	]>[-<vv
	if plus
		find the start of the tape
		vvv
		+[-<+]->
		check right for a tape head
		-[+>-]+
		increment
		v+^
		find the pc again
		+[-<+]-
		<^^^
		+[->+]-
		>-[+>-]+

	^^>]<
	
	vv
>
+]-