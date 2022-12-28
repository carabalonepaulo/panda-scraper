require 'json'
require 'mechanize'

agent = Mechanize.new
agent.user_agent_alias  = 'Windows Mozilla'

page = agent.get('https://www.panda-novel.com/browsenovel')

file = File.open('./src/panda/tag.rs', 'w')
page
  .search('.filter-list > li > a:nth-child(1)')
  .map { |a| { title: a.text, value: a[:href].sub('/browsenovel/', '') } }
  .each do |tag|
    title = tag[:title].gsub(/(-|\s)+/, '_').upcase

    file.write("pub const #{title}: &str = \"#{tag[:value]}\";\n")
  end
file.close