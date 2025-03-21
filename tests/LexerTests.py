import os
import subprocess
 
 # Path to the folder containing your C files
folder_path = rf'C:\Users\noam\Documents\Compiler\C-Compiler\write_a_c_compiler\stage_5\valid'

# Path to your lexer executable
lexer_exe = rf'C:\Users\noam\Documents\Compiler\C-Compiler\Compiler_org\target\debug\Compiler_org.exe'

# Get all the C files in the folder
c_files = [f for f in os.listdir(folder_path) if f.endswith('.c')]
files_max , files_curr = len(c_files) , 0

# Run the lexer on each C file
for c_file in c_files:
    file_path = folder_path + '\\'+  c_file
    result = ""
    print("-------- Tests for -  ",  c_file , "----------------\n\n")


    try:
        result = subprocess.run([lexer_exe, file_path], capture_output=True, text=True, check=True)
        print(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Error processing {c_file}: {e.stderr}")
        print(c_file , " Couldn't work")

    if result:
        with open(rf"..\tmps\tmp.s" , 'w') as file:
            file.writelines(result.stdout)

        p = subprocess.run(rf"gcc -m64 ..\tmps\tmp.s -o ..\executables\tmp.exe" , capture_output=True)
        if p.stderr:
            break

        try:
            p = subprocess.run(rf"..\executables\tmp.exe" , capture_output=True)
        except Exception as e:
            print(e)
        else:
            print("Seems good !\n")
            files_curr +=1
    print("------------- To The Next one !! ---------- \n\n\n")

print("At total ", files_curr , " tests were passed out of - " , files_max)