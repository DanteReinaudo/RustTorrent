import * as React from "react";
import Chart from "react-apexcharts";
import {createTheme, useColorScheme} from '@mui/material/styles';
import {colors} from "@mui/material";


const theme = createTheme({
    palette: {
        primary: {
            light: '#757ce8',
            main: '#303f9b',
            dark: '#283749',
            contrastText: '#fff',
        },
        secondary: {
            light: 'rgba(217,115,157,0.56)',
            main: 'rgba(224,101,148,0.88)',
            dark: '#af0c65',
            contrastText: '#000',
        },
    },
});


export default class BarGraph extends React.PureComponent {
    constructor(props) {
        super(props);

        const categories = props.props.xaxis
        const data = props.props.data

        this.state = {
            options: {
                bar: {
                    horizontal: false,
                    borderRadius: 70,
                    columnWidth: '70%',
                    barHeight: '70%',
                    distributed: false,
                    rangeBarOverlap: true,
                    rangeBarGroupRows: false,
                    colors: {
                        ranges: [{
                            from: 0,
                            to: 0,
                            color: theme.palette.secondary
                        }],
                        backgroundBarOpacity: 1,
                        backgroundBarRadius: 0,
                    }
                },
                chart: {
                    id: "basic-bar"
                },
                xaxis: {
                    categories: categories,
                    title: {
                        text: 'Tiempo'
                    }
                },
                yaxis: {
                    title: {
                        text: 'Cantidad'
                    }
                },
                dataLabels: {
                   enabled: false,
                   textAnchor: 'start',
                   style: {
                       colors: ['rgb(255,255,255)']
                   }
               },
                palette: {
                    primary: {
                        main: theme.palette.primary,
                    },
                    secondary: {
                        main: theme.palette.secondary,
                    },
                },
                fill: {
                    opacity: 1
                },
                labels: {
                    style: {
                        colors: theme.palette.secondary,
                        fontSize: '20px'
                    }
                },
            },

            series: [
                {
                    name: "Cantidad ",
                    data: data
                }
            ]
        };
    }

    render() {
        return (
            <div className="app">
                <div className="row">
                    <div className="mixed-chart">
                        <Chart
                            options={this.state.options}
                            series={this.state.series}
                            type="bar"
                            width="30%"
                        />
                    </div>
                </div>
            </div>
        );
    }
}
