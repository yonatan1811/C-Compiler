import os
import subprocess

# Path to the folder containing your C files
folder_path = rf'C:\Users\noam\Documents\Compiler\C-Compiler\write_a_c_compiler\stage_2\valid'

# Path to your lexer executable
lexer_exe = 'Parser.exe'

# Get all the C files in the folder
c_files = [f for f in os.listdir(folder_path) if f.endswith('.c')]

# Run the lexer on each C file
for c_file in c_files:
    file_path = os.path.join(folder_path, c_file)
    
    result = ""
    # Run the lexer via subprocess, passing the file path as an argument
    try:
        result = subprocess.run([lexer_exe, file_path], capture_output=True, text=True, check=True)
        print(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Error processing {c_file}: {e.stderr}")
        print(c_file , " Couldn't work")

    if result:
        with open("tmp.s" , 'w') as file:
            file.writelines(result.stdout)

        p = subprocess.run("gcc -m64 tmp.s -o tmp" , capture_output=True)
        if p.stderr:
            print(p.stderr)
            print("Command couldn't work")
            break

        try:
            p = subprocess.run("tmp.exe" , capture_output=True)
            print(p.stdout , p.stderr)
        except Exception as e:
            print(e)
        



