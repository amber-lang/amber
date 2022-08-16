age=22;
if [ $(echo $age '>=' 18 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
echo "You can drive"
else
echo "You cannot drive"
fi