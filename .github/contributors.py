import sys
import requests

try:
    filename = sys.argv[1]
except IndexError:
    print("Usage: contributors.py file.md")
    exit(1)

repo_owner = "rochacbruno"
repo_name = "marmite"
contributors_url = (
    f"https://api.github.com/repos/{repo_owner}/{repo_name}/contributors"
)

response = requests.get(contributors_url)
if response.status_code != 200:
    raise Exception(f"Failed to fetch contributors: {response.status_code}")

contributors = response.json()
contributors_sorted = reversed(
    sorted(contributors, key=lambda user: user["contributions"])
)

with open(filename, "w") as file:
    file.write("# Contributors\n\n")
    file.write('<div class="grid" style="display: flex;flex-flow:wrap;">\n')
    for i, contributor in enumerate(contributors_sorted):
        # # Start a new grid after every 5 contributors
        # if i % 5 == 0:
        #     if i != 0:
        #         file.write("</div>\n")
        #     file.write('<div class="grid" style="display: flex;">\n')

        username = contributor["login"]
        profile_url = contributor["html_url"]
        avatar_url = contributor["avatar_url"]
        contributions = contributor["contributions"]

        file.write('    <article style="width: 250px;text-align: center;">\n')
        file.write(
            '       <header style="text-align: center;">'
            f'<a href="{profile_url}" target="_blank">'
            f"{username}</a></header>\n"
        )
        file.write(
            f'       <a href="{profile_url}" target="_blank">'
            f'<img src="{avatar_url}" style="width: 100px;"></a>\n'
        )
        file.write(
            '       <footer style="text-align: center;">'
            f"{contributions} commits</footer>\n"
        )
        file.write("    </article>\n")

    file.write("</div>\n")

print("contributors.md file has been generated!")
